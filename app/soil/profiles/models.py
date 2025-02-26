import datetime
from geoalchemy2 import Geometry
from pydantic import model_validator
from sqlmodel import (
    SQLModel,
    Field,
    UniqueConstraint,
    Relationship,
    Column,
    JSON,
)
from typing import TYPE_CHECKING, Any
from uuid import uuid4, UUID
from app.areas.models import Area
from app.config import config
from sqlalchemy.sql import func
from app.utils.validators import (
    convert_wkb_to_x_y,
    convert_x_y_to_wkt,
    resize_images,
)

if TYPE_CHECKING:
    from app.soil.types.models import SoilType


class SoilProfileBase(SQLModel):
    name: str = Field(index=True)
    # profile_iterator: int = Field(
    #     description=(
    #         "The ID given by the scientist to the soil profile and forms part "
    #         "of the field ID. ie. 1 will become the 1 in BF01"
    #     ),
    #     default=None,
    #     index=True,
    # )
    gradient: str | None = Field(
        default=None,
        index=True,
        nullable=False,
    )
    description_horizon: Any = Field(default=[], sa_column=Column(JSON))

    weather: str | None = Field(
        default=None,
    )
    topography: str | None = Field(
        default=None,
    )
    vegetation_type: str | None = Field(
        default=None,
    )
    aspect: str | None = Field(
        default=None,
    )
    parent_material: float | None = Field(
        default=None,
    )
    lythology_surficial_deposit: str | None = Field(
        default=None,
    )
    created_on: datetime.datetime | None = Field(
        default=None,
        nullable=True,
        index=True,
    )
    soil_type_id: UUID = Field(
        foreign_key="soiltype.id",
        default=None,
        index=True,
    )
    area_id: UUID = Field(
        foreign_key="area.id",
        default=None,
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
    soil_diagram: str | None = Field(
        default=None,
        description="Base64 encoded diagram of the soil profile",
    )
    photo: str | None = Field(
        default=None,
        description="Base64 encoded photo of the soil profile",
    )

    class Config:
        arbitrary_types_allowed = True


class SoilProfile(SoilProfileBase, table=True):
    __table_args__ = (
        UniqueConstraint("id"),
        UniqueConstraint(
            # "profile_iterator",
            "area_id",
            "gradient",
            name="unique_profile",
        ),
    )
    # iterator: int = Field(
    #     default=None,
    #     nullable=False,
    #     index=True,
    # )
    id: UUID = Field(
        default_factory=uuid4,
        index=True,
        primary_key=True,
        nullable=False,
    )
    geom: Any = Field(
        default=None, sa_column=Column(Geometry("POINTZ", srid=config.SRID))
    )

    soil_type: "SoilType" = Relationship(
        back_populates="soil_profiles",
        sa_relationship_kwargs={"lazy": "selectin"},
    )

    area: Area = Relationship(
        back_populates="soil_profiles",
        sa_relationship_kwargs={"lazy": "selectin"},
    )


class SoilProfileRead(SoilProfileBase):
    id: UUID
    geom: Any | None = None
    coord_x: float | None = None
    coord_y: float | None = None
    coord_z: float | None = None
    coord_srid: int | None = None

    latitude: float | None = None
    longitude: float | None = None

    _convert_wkb_to_x_y = model_validator(mode="after")(convert_wkb_to_x_y)


class GenericNameIDModel(SQLModel):
    name: str
    id: UUID


class SoilProfileReadWithArea(SoilProfileRead):
    id: UUID
    area: GenericNameIDModel
    soil_type: GenericNameIDModel


class SoilProfileCreate(SoilProfileBase):
    coord_x: float | None
    coord_y: float | None
    coord_z: float | None

    geom: Any | None = None

    name: str | None = None

    _convert_x_y_to_wkt = model_validator(mode="after")(convert_x_y_to_wkt)
    _resize_images = model_validator(mode="after")(resize_images)


class SoilProfileUpdate(SoilProfileBase):
    coord_x: float | None
    coord_y: float | None
    coord_z: float | None

    geom: Any | None = None

    _convert_x_y_to_wkt = model_validator(mode="after")(convert_x_y_to_wkt)
    _resize_images = model_validator(mode="after")(resize_images)
