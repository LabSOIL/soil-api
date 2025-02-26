from sqlmodel import SQLModel, Field, Column, Relationship, UniqueConstraint
from geoalchemy2 import Geometry
from uuid import uuid4, UUID
from typing import Any
from pydantic import model_validator
from typing import TYPE_CHECKING
import datetime
from app.config import config
from sqlalchemy.sql import func
from app.utils.validators import convert_wkb_to_x_y, convert_x_y_to_wkt

if TYPE_CHECKING:
    from app.areas.models import Area
    from app.plots.sensors.models import PlotSensorAssignments


class SensorBase(SQLModel):
    name: str | None = Field(default=None, index=True)
    description: str | None = Field(default=None)
    comment: str | None = Field(default=None)
    serial_number: str | None = Field(default=None)
    manufacturer: str | None = Field(default=None)
    last_updated: datetime.datetime = Field(
        default_factory=datetime.datetime.now,
        title="Last Updated",
        description="Date and time when the record was last updated",
        sa_column_kwargs={
            "onupdate": func.now(),
            "server_default": func.now(),
        },
    )


class Sensor(SensorBase, table=True):
    __table_args__ = (UniqueConstraint("id"),)
    # iterator: int = Field(
    #     default=None,
    #     nullable=False,
    #     index=True,
    # )
    # id: UUID = Field(
    #     primary_key=True,
    #     default_factory=uuid4,
    #     index=True,
    #     nullable=False,
    # )

    # geom: Any = Field(sa_column=Column(Geometry("POINTZ", srid=config.SRID)))
    area_id: UUID = Field(default=None, foreign_key="area.id")

    area: "Area" = Relationship(
        back_populates="sensors", sa_relationship_kwargs={"lazy": "selectin"}
    )
    data: list["SensorData"] = Relationship(
        back_populates="sensor",
        sa_relationship_kwargs={
            "lazy": "selectin",
            "cascade": "all,delete,delete-orphan",
        },
    )
    plot_link: list["PlotSensorAssignments"] = Relationship(
        back_populates="sensor",
        sa_relationship_kwargs={"lazy": "selectin"},
    )


class SensorDataBase(SQLModel):
    """Matching the data structure from the Tomst TMS data logger
    https://tomst.com/web/en/systems/tms/data/

    Where the date structure is as follows:
    0;31.10.2013 11:45;0;21.5625;22.0625;23.125;148;1;0

    0 	                index of the measure
    31.10.2013 11:45    date and time in UTC
    0 	                time zone
    21.5625 	        T1
    22.0625             T2
    23.125              T3
    148 	            soil moisture count (raw moisture data)
    1 	                shake
    0 	                errFlag
    """

    instrument_seq: int = Field(  # The iterator integer in the instrument
        index=True,
        nullable=False,
    )
    time_utc: datetime.datetime = Field(
        index=True,
        nullable=False,
    )
    # time_zone: int | None = Field(
    #     index=False,
    #     nullable=True,
    # )
    temperature_1: float | None = Field(
        index=True,
        nullable=True,
    )
    temperature_2: float | None = Field(
        index=True,
        nullable=True,
    )
    temperature_3: float | None = Field(
        index=True,
        nullable=True,
    )
    temperature_average: float | None = Field(
        index=True,
        nullable=True,
    )
    soil_moisture_count: float | None = Field(
        index=True,
        nullable=True,
    )
    shake: int | None = Field(
        index=False,
        nullable=True,
    )
    error_flat: int | None = Field(
        index=False,
        nullable=True,
    )

    sensor_id: UUID = Field(
        default=None,
        foreign_key="sensor.id",
        nullable=False,
        index=True,
    )

    last_updated: datetime.datetime = Field(
        default_factory=datetime.datetime.now,
        title="Last Updated",
        description="Date and time when the record was last updated",
        sa_column_kwargs={
            "onupdate": func.now(),
            "server_default": func.now(),
        },
    )


class SensorData(SensorDataBase, table=True):
    __table_args__ = (
        UniqueConstraint("id"),
        UniqueConstraint("instrument_seq", "sensor_id"),
        UniqueConstraint("time_utc", "sensor_id"),
    )

    id: UUID = Field(
        default_factory=uuid4,
        primary_key=True,
        index=True,
        nullable=False,
    )

    sensor: Sensor = Relationship(
        back_populates="data",
        sa_relationship_kwargs={"lazy": "selectin"},
    )


class SensorDataRead(SensorDataBase):
    id: UUID
    sensor: Any


class SensorDataCreate(SensorDataBase):
    pass


class SensorRead(SensorBase):
    id: UUID
    area_id: UUID
    geom: Any | None
    geom: Any | None = None
    coord_x: float | None = None
    coord_y: float | None = None
    coord_z: float | None = None
    coord_srid: int | None = None
    latitude: float | None = None
    longitude: float | None = None

    area: Any | None = None

    _convert_wkb_to_x_y = model_validator(mode="after")(convert_wkb_to_x_y)


class SensorReadWithData(SensorRead):
    data: list[Any] | None = None


class SensorDataSummary(SQLModel):
    start_date: datetime.datetime | None = None
    end_date: datetime.datetime | None = None
    qty_records: int | None = None


class SensorReadWithDataSummary(SensorRead):
    data: SensorDataSummary


class SensorReadWithDataSummaryAndPlot(SensorRead):
    data: SensorDataSummary | None
    temperature_plot: list[SensorDataRead] | None = None


class SensorCreate(SensorBase):
    area_id: UUID
    coord_y: float
    coord_x: float
    coord_z: float

    latitude: float | None = None
    longitude: float | None = None

    # geom: Any | None = None
    data_base64: str | None = None  # Base64 encoded CSV data

    _convert_x_y_to_wkt = model_validator(mode="after")(convert_x_y_to_wkt)


class SensorUpdate(SensorCreate):
    pass
