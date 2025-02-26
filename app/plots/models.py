import datetime
from geoalchemy2 import Geometry
from pydantic import model_validator
from sqlmodel import (
    SQLModel,
    Field,
    Column,
    UniqueConstraint,
    Relationship,
    Enum,
)
from typing import Any, TYPE_CHECKING
from uuid import UUID, uuid4
from app.config import config
from sqlalchemy.sql import func
from app.transects.models.nodes import TransectNode
from app.transects.models.transects import Transect
import enum
from app.utils.validators import (
    convert_wkb_to_x_y,
    convert_x_y_to_wkt,
    resize_image,
    empty_string_to_none,
)

if TYPE_CHECKING:
    from app.areas.models import Area
    from app.plots.samples.models import PlotSample
from app.plots.sensors.models import PlotSensorAssignments


class GradientChoices(str, enum.Enum):
    flat = "flat"
    slope = "slope"


class PlotBase(SQLModel):
    name: str = Field(
        index=True,
        nullable=False,
    )
    # plot_iterator: int = Field(
    #     description=(
    #         "The ID given by the scientist to the plot and forms part "
    #         "of the field ID. ie. 1 will become the 1 in BF01"
    #     ),
    #     default=None,
    #     index=True,
    # )
    area_id: UUID = Field(
        nullable=False,
        index=True,
        foreign_key="area.id",
    )
    gradient: GradientChoices = Field(
        sa_column=Column(
            Enum(GradientChoices),
            nullable=False,
            index=True,
        ),
    )
    vegetation_type: str | None = Field(
        default=None,
    )
    topography: str | None = Field(
        default=None,
    )
    aspect: str | None = Field(
        default=None,
    )
    created_on: datetime.date | None = Field(
        None,
    )
    weather: str | None = Field(
        default=None,
    )
    lithology: str | None = Field(
        default=None,
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
    image: str | None = Field(
        default=None,
        description="Base64 encoded image",
    )


class Plot(PlotBase, table=True):
    __table_args__ = (
        UniqueConstraint("id"),
        UniqueConstraint(
            # "plot_iterator",
            "area_id",
            "gradient",
            name="unique_plot",
        ),
        UniqueConstraint(
            "name",
            name="unique_plot_name",
        ),
    )
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
    geom: Any = Field(
        default=None, sa_column=Column(Geometry("POINTZ", srid=config.SRID))
    )
    area: "Area" = Relationship(
        sa_relationship_kwargs={"lazy": "selectin"},
        back_populates="plots",
    )
    samples: list["PlotSample"] = Relationship(
        back_populates="plot",
        sa_relationship_kwargs={
            "lazy": "selectin",
            "cascade": "all,delete,delete-orphan",
        },
    )
    transects: list[Transect] = Relationship(
        back_populates="nodes",
        sa_relationship_kwargs={
            "lazy": "selectin",
        },
        link_model=TransectNode,
    )
    sensor_link: list[PlotSensorAssignments] = Relationship(
        back_populates="plot",
        sa_relationship_kwargs={
            "lazy": "selectin",
            "cascade": "all,delete,delete-orphan",
        },
    )


class PlotRead(PlotBase):
    id: UUID
    geom: Any | None = None
    coord_x: float | None = None
    coord_y: float | None = None
    coord_z: float | None = None
    coord_srid: int | None = None
    latitude: float | None = None
    longitude: float | None = None

    area: Any
    name: str | None = None

    _convert_wkb_to_x_y = model_validator(mode="after")(convert_wkb_to_x_y)


class NestedAreaWithProject(SQLModel):
    # Write a copy of AreaBase here to avoid circular imports
    description: str | None
    id: UUID | None
    name: str | None
    project_id: UUID | None
    project: Any | None


class PlotReadWithArea(PlotRead):
    area: NestedAreaWithProject


class SensorDistance(SQLModel):
    id: UUID
    distance: float
    name: str | None = None
    elevation_difference: float


class PlotReadWithSamples(PlotReadWithArea):
    samples: list[Any] = []
    area: Any
    transects: list[Any] = []
    sensors: list[SensorDistance] = []
    sensor_link: list[Any] = []


class PlotCreate(PlotBase):
    area_name: str | None = None  # Endpoint func will find area ID by name
    area_id: UUID | None = None  # Can be None to allow discovery by name

    coord_x: float | None
    coord_y: float | None
    coord_z: float | None = None

    geom: Any | None = None

    name: str | None = None  # Set null to allow endpoint func to gen. name

    # Validators
    _convert_x_y_to_wkt = model_validator(mode="after")(convert_x_y_to_wkt)
    _resize_image = model_validator(mode="after")(resize_image)
    _handle_empty_string = model_validator(mode="before")(empty_string_to_none)


class PlotUpdate(PlotBase):
    area_name: str | None = None  # Endpoint func will find area ID by name
    area_id: UUID | None = None  # Can be None to allow discovery by name

    coord_x: float | None
    coord_y: float | None
    coord_z: float | None

    geom: Any | None = None

    name: str | None = None  # Set null to allow endpoint func to gen. name

    # Validators
    _convert_x_y_to_wkt = model_validator(mode="after")(convert_x_y_to_wkt)
    _resize_image = model_validator(mode="after")(resize_image)
    _handle_empty_string = model_validator(mode="before")(empty_string_to_none)


class PlotUpdateBatch(PlotUpdate):
    id: UUID
