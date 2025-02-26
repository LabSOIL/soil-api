from sqlmodel import SQLModel, Field, Relationship, UniqueConstraint
from uuid import UUID, uuid4
import datetime
from sqlalchemy.sql import func
from typing import TYPE_CHECKING, Any
from app.transects.models.nodes import TransectNode
from pydantic import model_validator
from app.utils.validators import convert_x_y_to_wkt

if TYPE_CHECKING:
    from app.plots.models import Plot
    from app.areas.models import Area


class TransectBase(SQLModel):
    name: str | None = Field(
        default=None,
        index=True,
        nullable=True,
    )
    description: str | None = Field(
        default=None,
    )
    date_created: datetime.datetime | None = Field(
        default=None,
        nullable=True,
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
    area_id: UUID = Field(
        nullable=False,
        index=True,
        foreign_key="area.id",
    )


class Transect(TransectBase, table=True):
    __table_args__ = (UniqueConstraint("id"),)
    # iterator: int = Field(
    #     default=None,
    #     nullable=False,
    #     index=True,
    # )
    id: UUID = Field(
        default_factory=uuid4,
        primary_key=True,
        index=True,
        nullable=False,
    )

    nodes: list["Plot"] = Relationship(
        back_populates="transects",
        sa_relationship_kwargs={"lazy": "selectin"},
        link_model=TransectNode,
    )
    area: "Area" = Relationship(
        back_populates="transects",
        sa_relationship_kwargs={"lazy": "selectin"},
    )


class PlotSimple(SQLModel):
    # We only need the id of the plot
    id: UUID
    name: str | None = None
    geom: Any
    coord_x: float | None = None
    coord_y: float | None = None
    coord_z: float | None = None
    latitude: float | None = None
    longitude: float | None = None
    coord_srid: int | None = None

    _convert_x_y_to_wkt = model_validator(mode="after")(convert_x_y_to_wkt)


class TransectRead(TransectBase):
    id: UUID
    nodes: list[PlotSimple]
    area: Any


class TransectCreate(TransectBase):
    nodes: list[PlotSimple]


class TransectUpdate(TransectBase):
    pass
