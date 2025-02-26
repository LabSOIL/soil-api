from sqlmodel import SQLModel, Field, Relationship, UniqueConstraint
from uuid import uuid4, UUID
from typing import Any, TYPE_CHECKING
import datetime
from pydantic import field_validator


if TYPE_CHECKING:
    from app.plots.models import Plot
    from app.sensors.models import Sensor


class PlotSensorAssignmentBase(SQLModel):
    date_from: datetime.datetime = Field(
        default_factory=datetime.datetime.now,
        nullable=False,
    )
    date_to: datetime.datetime = Field(
        default_factory=datetime.datetime.now,
        nullable=False,
    )
    depth_cm: int = Field(
        description=(
            "Defines the depth of the sensor placement, as to group "
            "others together if need be"
        ),
        nullable=False,
    )

    plot_id: UUID = Field(
        foreign_key="plot.id",
        nullable=False,
    )
    sensor_id: UUID | None = Field(
        foreign_key="sensor.id",
        nullable=False,
    )


class PlotSensorAssignmentsCreate(PlotSensorAssignmentBase):
    pass


class PlotSensorAssignmentsRead(PlotSensorAssignmentBase):
    id: UUID
    plot: Any
    sensor: Any


class PlotSensorAssignmentsUpdate(PlotSensorAssignmentBase):
    pass


class PlotSensorAssignments(PlotSensorAssignmentBase, table=True):
    __table_args__ = (UniqueConstraint("id"),)
    # iterator: int = Field(
    #     default=None,
    #     nullable=False,
    #     index=True,
    # )
    id: UUID = Field(
        primary_key=True,
        default_factory=uuid4,
        index=True,
        nullable=False,
    )

    plot: "Plot" = Relationship(
        back_populates="sensor_link",
        sa_relationship_kwargs={"lazy": "selectin"},
    )
    sensor: "Sensor" = Relationship(
        back_populates="plot_link",
        sa_relationship_kwargs={"lazy": "selectin"},
    )
