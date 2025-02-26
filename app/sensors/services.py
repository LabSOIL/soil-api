from app.sensors.models import (
    SensorRead,
    SensorReadWithData,
    Sensor,
    SensorCreate,
    SensorUpdate,
    SensorDataBase,
    SensorData,
    # ClosestFeature,
)
from app.db import get_session, AsyncSession
from fastapi import Depends, Query, Response, HTTPException
from sqlmodel import select
from uuid import UUID
from app.crud import CRUD
from app.utils.funcs import decode_base64
from datetime import datetime
import numpy as np
from lttb import downsample


# from sqlalchemy.sql import func, text
# from app.soil.profiles.models import SoilProfile
# from app.plots.models import Plot

crud = CRUD(Sensor, SensorRead, SensorCreate, SensorUpdate)


def simplify_sensor_data_lttb(
    data: list[SensorDataBase], target_points: int = 100
) -> list[SensorDataBase]:
    """
    Simplifies the sensor data using the Largest-Triangle-Three-Buckets (LTTB)
    algorithm, applied to each variable that needs downsampling, while
    preserving the rest of the data.

    Args:
        data: List of SensorDataRead containing time series sensor data.
        target_points: The target number of data points after simplification.

    Returns:
        List of SensorDataRead with simplified data.
    """
    if len(data) <= target_points:
        return data  # Return the data as-is if it's already small enough

    # Prepare the time data
    times = np.array(
        [d.time_utc.timestamp() for d in data]
    )  # Convert time_utc to timestamps

    # Downsample the time values first
    time_data = np.stack(
        [times, times], axis=-1
    )  # LTTB expects a 2-column array, so duplicate the x-axis
    downsampled_time_data = downsample(time_data, target_points)
    downsampled_times = downsampled_time_data[
        :, 0
    ]  # Extract the downsampled timestamps

    # Helper to map downsampled times to original indices
    downsampled_set = set(downsampled_times)

    # Filter the original data by the downsampled times
    downsampled_original_data = [
        d for d in data if d.time_utc.timestamp() in downsampled_set
    ]

    # Now, apply the LTTB algorithm for each y-axis variable separately
    def downsample_field(field_data: np.ndarray) -> np.ndarray:
        y_data = np.stack(
            [times, field_data], axis=-1
        )  # Combine the time and y-value
        downsampled_y_data = downsample(y_data, target_points)
        return downsampled_y_data[:, 1]  # Return only the downsampled y-values

    # Apply downsampling for each of the fields
    temp1 = np.array([d.temperature_1 for d in data])
    temp2 = np.array([d.temperature_2 for d in data])
    temp3 = np.array([d.temperature_3 for d in data])
    temp_avg = np.array([d.temperature_average for d in data])
    moisture = np.array([d.soil_moisture_count for d in data])

    downsampled_temp1 = downsample_field(temp1)
    downsampled_temp2 = downsample_field(temp2)
    downsampled_temp3 = downsample_field(temp3)
    downsampled_temp_avg = downsample_field(temp_avg)
    downsampled_moisture = downsample_field(moisture)

    # Build the simplified data by updating the downsampled fields while
    # preserving other fields
    simplified_data = []
    for i, d in enumerate(downsampled_original_data):
        simplified_data.append(
            SensorDataBase(
                instrument_seq=d.instrument_seq,
                time_utc=d.time_utc,  # Keep the downsampled time
                time_zone=d.time_zone,
                temperature_1=downsampled_temp1[i],
                temperature_2=downsampled_temp2[i],
                temperature_3=downsampled_temp3[i],
                temperature_average=downsampled_temp_avg[i],
                soil_moisture_count=downsampled_moisture[i],
                shake=d.shake,
                error_flat=d.error_flat,
            )
        )

    return simplified_data


async def get_count(
    response: Response,
    filter: str = Query(None),
    range: str = Query(None),
    sort: str = Query(None),
    session: AsyncSession = Depends(get_session),
):
    count = await crud.get_total_count(
        response=response,
        sort=sort,
        range=range,
        filter=filter,
        session=session,
    )

    return count


async def get_data(
    filter: str = Query(None),
    sort: str = Query(None),
    range: str = Query(None),
    session: AsyncSession = Depends(get_session),
):
    res = await crud.get_model_data(
        sort=sort,
        range=range,
        filter=filter,
        session=session,
    )

    return res


async def get_one(
    sensor_id: UUID,
    low_resolution: bool = Query(False),
    session: AsyncSession = Depends(get_session),
):
    # Fetch the sensor by ID
    res = await crud.get_model_by_id(model_id=sensor_id, session=session)

    if not res:
        raise HTTPException(
            status_code=404, detail=f"ID: {sensor_id} not found"
        )

    res = SensorReadWithData.model_validate(res)

    if low_resolution:
        res.data = simplify_sensor_data_lttb(res.data)

    # # Custom SQL query to get all plots and their distances to the sensor
    # stmt_plots = (
    #     select(
    #         Plot,
    #         func.st_distance(Sensor.geom, Plot.geom).label("distance"),
    #         (func.st_z(Sensor.geom) - func.st_z(Plot.geom)).label(
    #             "elevation_difference"
    #         ),
    #         text("'plot' AS type"),  # No label(), alias directly in text()
    #     )
    #     .where(Sensor.id == sensor_id)
    #     .where(Plot.area_id == res.area_id)
    #     .order_by(func.st_distance(Sensor.geom, Plot.geom))
    # )

    # Custom query to get all soil profiles and their distances to the sensor
    # stmt_soil_profiles = (
    #     select(
    #         SoilProfile,
    #         func.st_distance(
    # Sensor.geom, SoilProfile.geom).label("distance"),
    #         (func.st_z(Sensor.geom) - func.st_z(SoilProfile.geom)).label(
    #             "elevation_difference"
    #         ),
    #         text("'soil_profile' AS type"),
    #     )
    #     .where(Sensor.id == sensor_id)
    #     .where(SoilProfile.area_id == res.area_id)
    #     .order_by(func.st_distance(Sensor.geom, SoilProfile.geom))
    # )

    # Execute the SQL queries
    # result_plots = await session.exec(stmt_plots)
    # result_soil_profiles = await session.exec(stmt_soil_profiles)

    # plots = result_plots.fetchall()
    # profiles = result_soil_profiles.fetchall()

    # Combine both lists of plots and soil profiles
    closest_features = []
    # for features in [plots, profiles]:
    #    for feature, distance, elevation_difference, feature_type in features:
    #         closest_features.append(
    #             ClosestFeature(
    #                 id=feature.id,
    #                 name=feature.name,
    #                 distance=distance,
    #                 elevation_difference=elevation_difference,
    #                 type=feature_type,
    #             )
    #         )

    # Add the closest features to the sensor response, add the closest 10
    res.closest_features = sorted(closest_features, key=lambda x: x.distance)[
        :10
    ]

    return res


def ingest_csv_data(
    sensor_data: bytes,
    sensor_id: UUID,
) -> list[SensorData]:

    lines = sensor_data.decode("utf-8").split("\n")
    objs = []
    for line in lines:
        if line:
            data = line.split(";")

            time_utc = datetime.strptime(data[1], "%Y.%m.%d %H:%M")
            sensor_data_obj = SensorData(
                instrument_seq=int(data[0]),
                time_utc=time_utc,
                time_zone=int(data[2]),
                temperature_1=float(data[3]),
                temperature_2=float(data[4]),
                temperature_3=float(data[5]),
                temperature_average=(
                    (float(data[3]) + float(data[4]) + float(data[5])) / 3
                ),
                soil_moisture_count=int(data[6]),
                shake=int(data[7]),
                error_flat=int(data[8]),
                sensor_id=sensor_id,
            )

            objs.append(sensor_data_obj)

    return objs


async def create_one(
    sensor: SensorCreate,
    session: AsyncSession = Depends(get_session),
) -> Sensor:

    sensor_obj = Sensor.model_validate(sensor)

    session.add(sensor_obj)
    await session.commit()
    await session.refresh(sensor_obj)

    if sensor.data_base64:
        # Decode Base64 CSV data and add it to SensorData table
        sensor_data, filetype = decode_base64(sensor.data_base64)
        if filetype != "csv":
            raise HTTPException(
                status_code=400,
                detail="Only CSV files are supported",
            )

        data_objs = ingest_csv_data(sensor_data, sensor_obj.id)
        session.add_all(data_objs)

    await session.commit()
    await session.refresh(sensor_obj)

    return sensor_obj


async def update_one(
    sensor_update: SensorUpdate,
    sensor: SensorRead = Depends(get_one),
    session: AsyncSession = Depends(get_session),
) -> SensorRead:

    update_data = sensor_update.model_dump(exclude_unset=True)

    sensor.sqlmodel_update(update_data)
    session.add(sensor)

    if sensor_update.data_base64:
        # Decode Base64 CSV data and add it to SensorData table
        sensor_data, filetype = decode_base64(sensor_update.data_base64)
        if filetype != "csv":
            raise HTTPException(
                status_code=400,
                detail="Only CSV files are supported",
            )

        # Delete first all the data for this sensor
        query = select(SensorData).where(SensorData.sensor_id == sensor.id)
        res = await session.exec(query)
        res_objs = res.all()

        for obj in res_objs:
            await session.delete(obj)

        await session.commit()

        # Now add the new data
        data_objs = ingest_csv_data(sensor_data, sensor.id)
        session.add_all(data_objs)

    await session.commit()
    await session.refresh(sensor)

    return sensor
