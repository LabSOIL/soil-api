from sqlmodel import (
    SQLModel,
    Field,
    UniqueConstraint,
    Relationship,
)
from uuid import uuid4, UUID
from SecretColors import Palette
from typing import TYPE_CHECKING
import datetime
from sqlalchemy.sql import func

if TYPE_CHECKING:
    from app.areas.models import Area
    from app.instruments.models.experiment import InstrumentExperiment


class ProjectBase(SQLModel):
    name: str
    description: str | None = None
    color: str = Field(default_factory=Palette().random)
    last_updated: datetime.datetime = Field(
        default_factory=datetime.datetime.now,
        title="Last Updated",
        description="Date and time when the record was last updated",
        sa_column_kwargs={
            "onupdate": func.now(),
            "server_default": func.now(),
        },
    )


class Project(ProjectBase, table=True):
    __table_args__ = (
        UniqueConstraint("id"),
        UniqueConstraint("name"),
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

    areas: list["Area"] = Relationship(
        back_populates="project",
        sa_relationship_kwargs={"lazy": "selectin"},
    )
    instrument_experiments: list["InstrumentExperiment"] = Relationship(
        back_populates="project",
        sa_relationship_kwargs={"lazy": "selectin"},
    )


class ProjectCreate(ProjectBase):
    pass


class ProjectRead(ProjectBase):
    id: UUID


class ProjectUpdate(ProjectBase):
    pass
