from sqlmodel import SQLModel, Field, Relationship, UniqueConstraint
from uuid import UUID, uuid4
from typing import Any, List, Optional
import datetime
from sqlalchemy.sql import func
from app.instruments.channels.models import InstrumentExperimentChannel
from app.projects.models import Project


class InstrumentExperimentBase(SQLModel):
    name: Optional[str] = Field(default=None, index=True, nullable=True)
    date: Optional[datetime.datetime] = Field(default=None, nullable=True)
    description: Optional[str] = Field(default=None, nullable=True)
    filename: Optional[str] = Field(default=None, nullable=True)
    device_filename: Optional[str] = Field(
        description="The filename defined inside the text file as 'File'",
        default=None,
        nullable=True,
    )
    data_source: Optional[str] = Field(default=None, nullable=True)
    instrument_model: Optional[str] = Field(default=None, nullable=True)
    init_e: Optional[float] = Field(default=None, nullable=True)
    sample_interval: Optional[float] = Field(default=None, nullable=True)
    run_time: Optional[float] = Field(default=None, nullable=True)
    quiet_time: Optional[float] = Field(default=None, nullable=True)
    sensitivity: Optional[float] = Field(default=None, nullable=True)
    samples: Optional[int] = Field(
        description="Number of samples taken in the data file",
        default=None,
        nullable=True,
    )
    project_id: UUID | None = Field(
        default=None, nullable=True, index=True, foreign_key="project.id"
    )


class InstrumentExperiment(InstrumentExperimentBase, table=True):
    __table_args__ = (UniqueConstraint("id"),)
    # iterator: int = Field(
    #     default=None,
    #     nullable=False,
    #     index=True,
    # )
    id: UUID = Field(
        default_factory=uuid4,
        index=True,
        nullable=False,
        primary_key=True,
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

    channels: List["InstrumentExperimentChannel"] = Relationship(
        back_populates="experiment",
        sa_relationship_kwargs={
            "lazy": "selectin",
            "cascade": "all,delete,delete-orphan",
        },
    )
    project: Project = Relationship(
        sa_relationship_kwargs={"lazy": "selectin"},
        back_populates="instrument_experiments",
    )


class ChannelNoPoints(SQLModel):
    channel_name: str
    id: UUID
    baseline_values: list[Any] = []
    integral_results: list[Any] = []


class InstrumentExperimentRead(InstrumentExperimentBase):
    id: UUID
    channels: List[ChannelNoPoints] = []
    last_updated: datetime.datetime | None = None
    project: Any | None = None


class InstrumentExperimentUpdate(InstrumentExperimentBase):
    pass


class InstrumentExperimentCreate(InstrumentExperimentBase):
    data_base64: str = Field(...)
