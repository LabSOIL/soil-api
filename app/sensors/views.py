from fastapi import Depends, APIRouter, Query, Response, HTTPException, Body
from sqlmodel import select
from app.db import get_session, AsyncSession
from app.utils import decode_base64
from app.sensors.models import (
    Sensor,
    SensorRead,
    SensorUpdate,
    SensorCreate,
    SensorCreateFromGPX,
    SensorReadWithDataSummary,
    SensorReadWithDataSummaryAndPlot,
    SensorDataSummary,
    SensorData,
    SensorDataRead,
)
from uuid import UUID
from sqlalchemy import func
from datetime import timezone
import json
import gpxpy
import gpxpy.gpx
import csv

router = APIRouter()


## Sensor data


@router.get("/data/{sensordata_id}", response_model=SensorDataRead)
async def get_sensordata(
    session: AsyncSession = Depends(get_session),
    *,
    sensordata_id: UUID,
) -> SensorDataRead:
    """Get an sensordata by id"""
    res = await session.execute(
        select(SensorData).where(SensorData.id == sensordata_id)
    )
    sensordata = res.scalars().one_or_none()

    return sensordata


@router.get("/data", response_model=list[SensorDataRead])
async def get_all_sensordata(
    response: Response,
    session: AsyncSession = Depends(get_session),
    *,
    filter: str = Query(None),
    sort: str = Query(None),
    range: str = Query(None),
):
    """Get all sensordatas"""
    sort = json.loads(sort) if sort else []
    range = json.loads(range) if range else []
    filter = json.loads(filter) if filter else {}

    # Do a query to satisfy total count for "Content-Range" header
    count_query = select(func.count(SensorData.iterator))
    if len(filter):
        for field, value in filter.items():
            if field == "id" or field == "sensor_id":
                count_query = count_query.filter(
                    getattr(SensorData, field) == value
                )
            else:
                count_query = count_query.filter(
                    getattr(SensorData, field).like(f"%{str(value)}%")
                )
    total_count = await session.execute(count_query)
    total_count = total_count.scalar_one()

    query = select(SensorData)

    # Order by sort field params ie. ["name","ASC"]
    if len(sort) == 2:
        sort_field, sort_order = sort
        if sort_order == "ASC":
            query = query.order_by(getattr(SensorData, sort_field))
        else:
            query = query.order_by(getattr(SensorData, sort_field).desc())

    # Filter by filter field params ie. {"name":"bar"}
    if len(filter):
        for field, value in filter.items():
            if field == "id" or field == "sensor_id":
                query = query.filter(getattr(SensorData, field) == value)
            else:
                query = query.filter(
                    getattr(SensorData, field).like(f"%{str(value)}%")
                )

    if len(range) == 2:
        start, end = range
        query = query.offset(start).limit(end - start + 1)
    else:
        start, end = [0, total_count]

    # Execute query
    results = await session.execute(query)
    sensordatas = results.scalars().all()

    response.headers["Content-Range"] = (
        f"sensordata {start}-{end}/{total_count}"
    )

    return sensordatas


@router.post("/data", response_model=SensorDataRead)
async def create_sensordata(
    sensordata: SensorDataRead = Body(...),
    session: AsyncSession = Depends(get_session),
) -> SensorDataRead:
    """Creates an sensordata"""
    print(sensordata)
    sensordata = SensorData.from_orm(sensordata)
    session.add(sensordata)
    await session.commit()
    await session.refresh(sensordata)

    return sensordata


@router.put("/data/{sensordata_id}", response_model=SensorDataRead)
async def update_sensordata(
    sensordata_id: UUID,
    sensordata_update: SensorDataRead,
    session: AsyncSession = Depends(get_session),
) -> SensorDataRead:
    res = await session.execute(
        select(SensorData).where(SensorData.id == sensordata_id)
    )
    sensordata_db = res.scalars().one()
    sensordata_data = sensordata_update.dict(exclude_unset=True)
    if not sensordata_db:
        raise HTTPException(status_code=404, detail="SensorData not found")

    # Update the fields from the request
    for field, value in sensordata_data.items():
        if field in ["coord_x", "coord_y"]:
            # Don't process x/y, it's converted to geom in model validator
            continue

        print(f"Updating: {field}, {value}")
        setattr(sensordata_db, field, value)

    session.add(sensordata_db)
    await session.commit()
    await session.refresh(sensordata_db)

    return sensordata_db


@router.delete("/data/{sensordata_id}")
async def delete_sensordata(
    sensordata_id: UUID,
    session: AsyncSession = Depends(get_session),
    filter: dict[str, str] | None = None,
) -> None:
    """Delete an sensordata by id"""
    res = await session.execute(
        select(SensorData).where(SensorData.id == sensordata_id)
    )
    sensordata = res.scalars().one_or_none()

    if sensordata:
        await session.delete(sensordata)
        await session.commit()


## Sensor


@router.get("/{sensor_id}", response_model=SensorReadWithDataSummaryAndPlot)
async def get_sensor(
    session: AsyncSession = Depends(get_session),
    *,
    sensor_id: UUID,
) -> SensorRead:
    """Get an sensor by id"""

    query = (
        select(
            Sensor,
            func.count(SensorData.id).label("qty_records"),
            func.min(SensorData.time).label("start_date"),
            func.max(SensorData.time).label("end_date"),
        )
        .where(Sensor.id == sensor_id)
        .outerjoin(SensorData, Sensor.id == SensorData.sensor_id)
        .group_by(
            Sensor.id,
            Sensor.geom,
            Sensor.name,
            Sensor.description,
            Sensor.iterator,
        )
    )
    res = await session.execute(query)
    sensor = res.one_or_none()
    sensor_dict = sensor[0].dict() if sensor else {}

    # Do a query on the sensor data, at the moment this is raw, but should
    # probably be aggregated by day
    query = select(SensorData).where(SensorData.sensor_id == sensor_id)
    res = await session.execute(query)
    sensor_data = res.scalars().all()

    return SensorReadWithDataSummaryAndPlot(
        **sensor_dict,
        data=SensorDataSummary(
            qty_records=sensor[1] if sensor else None,
            start_date=sensor[2] if sensor else None,
            end_date=sensor[3] if sensor else None,
        ),
        temperature_plot=sensor_data,
    )


@router.get("", response_model=list[SensorReadWithDataSummary])
async def get_sensors(
    response: Response,
    session: AsyncSession = Depends(get_session),
    *,
    filter: str = Query(None),
    sort: str = Query(None),
    range: str = Query(None),
):
    """Get all sensors"""
    sort = json.loads(sort) if sort else []
    range = json.loads(range) if range else []
    filter = json.loads(filter) if filter else {}

    # Do a query to satisfy total count for "Content-Range" header
    count_query = select(func.count(Sensor.iterator))
    if len(filter):  # Have to filter twice for some reason? SQLModel state?
        for field, value in filter.items():
            if field == "id" or field == "area_id":
                count_query = count_query.filter(
                    getattr(Sensor, field) == value
                )
            else:
                count_query = count_query.filter(
                    getattr(Sensor, field).like(f"%{str(value)}%")
                )
    total_count = await session.execute(count_query)
    total_count = total_count.scalar_one()

    # Query for the quantity of records in SensorData that match the sensor as
    # well as the min and max of the time column
    query = (
        select(
            Sensor,
            func.count(SensorData.id).label("qty_records"),
            func.min(SensorData.time).label("start_date"),
            func.max(SensorData.time).label("end_date"),
        )
        .outerjoin(SensorData, Sensor.id == SensorData.sensor_id)
        .group_by(
            Sensor.id,
            Sensor.geom,
            Sensor.name,
            Sensor.description,
            Sensor.iterator,
        )
    )

    # Order by sort field params ie. ["name","ASC"]
    if len(sort) == 2:
        sort_field, sort_order = sort
        if sort_order == "ASC":
            query = query.order_by(getattr(Sensor, sort_field))
        else:
            query = query.order_by(getattr(Sensor, sort_field).desc())

    # Filter by filter field params ie. {"name":"bar"}
    if len(filter):
        for field, value in filter.items():
            if field == "id" or field == "area_id":
                query = query.filter(getattr(Sensor, field) == value)
            else:
                query = query.filter(
                    getattr(Sensor, field).like(f"%{str(value)}%")
                )

    if len(range) == 2:
        start, end = range
        query = query.offset(start).limit(end - start + 1)
    else:
        start, end = [0, total_count]  # For content-range header

    # Execute query
    results = await session.execute(query)
    sensors = results.all()
    # print(sensors)

    response.headers["Content-Range"] = f"sensors {start}-{end}/{total_count}"

    # Add the summary information for the data (instead of the full data)
    sensors_with_data = []
    for row in sensors:
        sensors_with_data.append(
            SensorReadWithDataSummary(
                **row[0].dict(),
                data=SensorDataSummary(
                    qty_records=row[1],
                    start_date=row[2],
                    end_date=row[3],
                ),
            )
        )

    return sensors_with_data


@router.post("/many", response_model=SensorRead)
async def create_many_sensors_from_gpx(
    sensor: SensorCreateFromGPX = Body(...),
    session: AsyncSession = Depends(get_session),
) -> None:
    """Creates a sensor from one or many GPX files"""

    for gpx_file in sensor.gpsx_files:
        # Read GPX file
        rawdata, dtype = decode_base64(gpx_file["src"])
        if dtype != "gpx":
            raise HTTPException(
                status_code=400,
                detail="Only GPX files are supported",
            )
        gpxdata = gpxpy.parse(rawdata)

        for waypoint in gpxdata.waypoints:
            # Use Sensorcreate to facilitate the lat/long to geom conversion

            obj = SensorCreate(
                name=waypoint.name,
                description=waypoint.description,
                comment=waypoint.comment,
                elevation=waypoint.elevation,
                coord_y=waypoint.latitude,
                coord_x=waypoint.longitude,
                time_recorded_at_utc=waypoint.time.replace(tzinfo=None),
                area_id=sensor.area_id,
            )

            session.add(Sensor.from_orm(obj))
        await session.commit()

    return Sensor.from_orm(obj)  # Just return one as react-admin expects one


@router.put("/{sensor_id}", response_model=SensorRead)
async def update_sensor(
    sensor_id: UUID,
    sensor_update: SensorUpdate,
    session: AsyncSession = Depends(get_session),
) -> SensorRead:
    res = await session.execute(select(Sensor).where(Sensor.id == sensor_id))
    sensor_db = res.scalars().one()
    sensor_data = sensor_update.dict(exclude_unset=True)
    if not sensor_db:
        raise HTTPException(status_code=404, detail="Sensor not found")

    # Update the fields from the request
    for field, value in sensor_data.items():
        if field in ["latitude", "longitude"]:
            # Don't process lat/lon, it's converted to geom in model validator
            continue
        if field == "instrumentdata":
            # Convert base64 to bytes, input should be csv, read and add rows
            # to sensor_data table with sensor_id
            rawdata, dtype = decode_base64(value)

            if dtype != "csv":
                raise HTTPException(
                    status_code=400,
                    detail="Only CSV files are supported",
                )
            # Treat the rawdata as a CSV file, read in the rows
            decoded = []
            for row in csv.reader(rawdata.decode("utf-8").splitlines()):
                decoded.append(row)

            print(rows)

        print(f"Updating: {field}, {value}")
        setattr(sensor_db, field, value)

    session.add(sensor_db)
    await session.commit()
    await session.refresh(sensor_db)

    return sensor_db


@router.delete("/batch", response_model=list[str])
async def delete_batch(
    ids: list[UUID],
    session: AsyncSession = Depends(get_session),
) -> list[str]:
    """Delete by a list of ids"""

    for id in ids:
        obj = await crud.get_model_by_id(model_id=id, session=session)
        if obj:
            await session.delete(obj)

    await session.commit()

    return [str(obj_id) for obj_id in ids]


@router.delete("/{sensor_id}")
async def delete_sensor(
    sensor_id: UUID,
    session: AsyncSession = Depends(get_session),
    filter: dict[str, str] | None = None,
) -> None:
    """Delete an sensor by id"""
    res = await session.execute(select(Sensor).where(Sensor.id == sensor_id))
    sensor = res.scalars().one_or_none()

    if sensor:
        await session.delete(sensor)
        await session.commit()
