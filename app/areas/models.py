from sqlmodel import SQLModel, Field, UniqueConstraint, Relationship
from uuid import uuid4, UUID
from typing import Any
from pydantic import model_validator
from typing import TYPE_CHECKING
from app.projects.models import Project
from sqlalchemy.sql import func
from app.sensors.models import Sensor
from app.plots.models import Plot
import datetime
from app.utils.validators import (
    convert_wkb_to_x_y,
    convert_wkb_to_json,
)

if TYPE_CHECKING:
    from app.soil.profiles.models import SoilProfile
    from app.transects.models.transects import Transect


class AreaBase(SQLModel):
    name: str = Field(
        default=None,
        index=True,
        nullable=False,
    )
    description: str | None = Field(
        default=None,
        nullable=True,
    )
    project_id: UUID = Field(
        nullable=False, index=True, foreign_key="project.id"
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


class Area(AreaBase, table=True):
    __table_args__ = (
        UniqueConstraint("id"),
        UniqueConstraint("name", "project_id", name="name_project_id"),
    )
    # iterator: int = Field(
    #     default=None,
    #     nullable=False,
    #     primary_key=True,
    #     index=True,
    # )
    id: UUID = Field(
        default_factory=uuid4,
        index=True,
        primary_key=True,
        nullable=False,
    )
    sensors: list["Sensor"] = Relationship(
        back_populates="area",
        sa_relationship_kwargs={"lazy": "selectin"},
    )

    project: Project = Relationship(
        sa_relationship_kwargs={"lazy": "selectin"},
        back_populates="areas",
    )

    plots: list["Plot"] = Relationship(
        back_populates="area",
        sa_relationship_kwargs={"lazy": "selectin"},
    )

    soil_profiles: list["SoilProfile"] = Relationship(
        back_populates="area",
        sa_relationship_kwargs={"lazy": "selectin"},
    )
    transects: list["Transect"] = Relationship(
        back_populates="area",
        sa_relationship_kwargs={"lazy": "selectin"},
    )


class GenericNameIDModel(SQLModel):
    name: str | None = None
    id: UUID


class GenericNameIDPointModel(GenericNameIDModel):
    geom: Any | None = None
    coord_x: float | None = None
    coord_y: float | None = None
    coord_z: float | None = None
    latitude: float | None = None
    longitude: float | None = None
    coord_srid: int | None = None

    _convert_wkb_to_x_y = model_validator(mode="after")(convert_wkb_to_x_y)


class TransectSimple(SQLModel):
    id: UUID
    name: str | None = None
    description: str | None = None
    area_id: UUID | None = None
    nodes: list[GenericNameIDPointModel] = []


class PlotSimple(GenericNameIDPointModel):
    samples: list[Any] = []


class AreaRead(AreaBase):
    id: UUID  # We use the UUID as the return ID
    geom: Any | None = None
    project: Project

    soil_profiles: list[GenericNameIDPointModel] = []
    plots: list[GenericNameIDPointModel] = []
    sensors: list[GenericNameIDPointModel] = []
    transects: list[TransectSimple] = []

    _convert_wkb_to_json = model_validator(mode="after")(convert_wkb_to_json)


class AreaCreate(AreaBase):
    pass


class AreaUpdate(AreaBase):
    pass
