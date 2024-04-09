from sqlmodel import SQLModel, Field, Column
from geoalchemy2 import Geometry
from uuid import uuid4, UUID
from typing import Any


class AreaBase(SQLModel):
    name: str = Field(default=None, index=True)
    description: str


class Area(AreaBase, table=True):
    id: int = Field(
        default=None,
        nullable=False,
        primary_key=True,
        index=True,
    )
    uuid: UUID = Field(
        default_factory=uuid4,
        index=True,
        nullable=False,
    )


class AreaRead(AreaBase):
    id: int
    uuid: UUID


class AreaCreate(AreaBase):
    pass
