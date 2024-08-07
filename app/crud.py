from app.db import get_session, AsyncSession
from fastapi import Depends, Response
from sqlmodel import select
from typing import Any
import json
from sqlalchemy.sql import func
from sqlalchemy import or_, cast, String
from uuid import UUID
from sqlmodel import SQLModel


class CRUD:
    def __init__(
        self,
        db_model: Any,
        db_model_read: Any,
        db_model_create: Any,
        db_model_update: Any,
    ):
        self.db_model = db_model
        self.db_model_read = db_model_read
        self.db_model_create = db_model_create
        self.db_model_update = db_model_update

    async def __call__(self, *args: Any, **kwds: Any) -> Any:
        pass

    @property
    def exact_match_fields(
        self,
    ) -> list[str]:
        """Returns a list of all the UUID fields in the model

        These cannot be performed with a likeness query and must have an
        exact match.

        """
        schema = self.db_model.model_json_schema()

        uuid_properties = []
        for prop_name, prop_details in schema["properties"].items():
            prop_type = prop_details.get("type")
            if isinstance(prop_type, list) and "string" in prop_type:
                any_of_types = prop_details.get("anyOf")
                if any_of_types:
                    for any_of_type in any_of_types:
                        if "string" in any_of_type.get("type", []):
                            uuid_properties.append(prop_name)
                            break
                elif (
                    "format" in prop_details
                    and prop_details["format"] == "uuid"
                ):
                    uuid_properties.append(prop_name)
            elif prop_type in ["string", "null"]:  # Allow case when optional
                if (
                    "format" in prop_details
                    and prop_details["format"] == "uuid"
                ):
                    uuid_properties.append(prop_name)

        return uuid_properties

    async def get_model_data(
        self,
        filter: str,
        sort: str,
        range: str,
        filter_models_to_join: list[SQLModel] = [],
        filter_fields_to_query: list[SQLModel] = [],
        session: AsyncSession = Depends(get_session),
    ) -> list:
        """Returns the data of a model with a filter applied

        Similar to the count query except returns the data instead of the count
        """

        sort = json.loads(sort) if sort else []
        range = json.loads(range) if range else []
        filter = json.loads(filter) if filter else {}

        query = select(self.db_model)

        if len(filter):
            for field, value in filter.items():
                if field == "q":
                    # If the field is 'q', do a full-text search on the
                    # searchable fields
                    or_conditions = []
                    for (
                        prop_name,
                        prop_details,
                    ) in self.db_model.model_json_schema()[
                        "properties"
                    ].items():

                        column = cast(
                            getattr(self.db_model, prop_name), String
                        )
                        or_conditions.append(
                            func.coalesce(column, "").ilike(f"%{str(value)}%")
                        )

                    # continue
                    if filter_fields_to_query and filter_models_to_join:
                        for model in filter_models_to_join:
                            query = query.join(model)
                        for field_to_query in filter_fields_to_query:
                            or_conditions.append(
                                field_to_query.ilike(f"%{value}%")
                            )

                    query = query.filter(or_(*or_conditions))
                    continue

                if field in self.exact_match_fields:
                    if isinstance(value, list):
                        # Combine multiple filters with OR
                        or_conditions = []
                        for v in value:
                            or_conditions.append(
                                getattr(self.db_model, field) == v
                            )

                        query = query.filter(or_(*or_conditions))
                    else:
                        # If it's not a list, apply a simple equality filter
                        query = query.filter(
                            getattr(self.db_model, field) == value
                        )
                else:
                    if isinstance(value, list):
                        or_conditions = []
                        for v in value:
                            or_conditions.append(
                                getattr(self.db_model, field).ilike(
                                    f"%{str(v)}%"
                                )
                            )

                        query = query.filter(or_(*or_conditions))
                    elif isinstance(value, bool):
                        if value is True:
                            query = query.filter(
                                getattr(self.db_model, field).has()
                            )
                        else:
                            query = query.filter(
                                ~getattr(self.db_model, field).has()
                            )
                    elif isinstance(value, int):
                        query = query.filter(
                            getattr(self.db_model, field) == value
                        )
                    else:
                        # Apply a LIKE filter for string matching
                        print(field)
                        query = query.filter(
                            func.coalesce(
                                getattr(self.db_model, field), ""
                            ).ilike(f"%{str(value)}%")
                        )

        if len(sort) == 2:
            sort_field, sort_order = sort
            if sort_order == "ASC":
                query = query.order_by(getattr(self.db_model, sort_field))
            else:
                query = query.order_by(
                    getattr(self.db_model, sort_field).desc()
                )

        if len(range):
            start, end = range
            query = query.offset(start).limit(end - start)

        print(query.compile(compile_kwargs={"literal_binds": True}))
        res = await session.exec(query)

        return res.all()

    async def get_total_count(
        self,
        response: Response,
        sort: str,
        range: str,
        filter: str,
        filter_models_to_join: list[SQLModel] = [],
        filter_fields_to_query: list[SQLModel] = [],
        session: AsyncSession = Depends(get_session),
    ) -> int:
        """Returns the count of a model with a filter applied"""

        filter = json.loads(filter) if filter else {}
        range = json.loads(range) if range else []

        query = select(func.count(self.db_model.iterator))
        if len(filter):
            for field, value in filter.items():
                if field == "q":
                    # If the field is 'q', do a full-text search on the
                    # searchable fields
                    or_conditions = []
                    for (
                        prop_name,
                        prop_details,
                    ) in self.db_model.model_json_schema()[
                        "properties"
                    ].items():

                        column = cast(
                            getattr(self.db_model, prop_name), String
                        )
                        or_conditions.append(
                            func.coalesce(column, "").ilike(f"%{str(value)}%")
                        )

                    # continue
                    if filter_fields_to_query and filter_models_to_join:
                        for model in filter_models_to_join:
                            query = query.join(model)
                        for field_to_query in filter_fields_to_query:
                            or_conditions.append(
                                field_to_query.ilike(f"%{value}%")
                            )

                    query = query.filter(or_(*or_conditions))
                    continue

                if field in self.exact_match_fields:
                    if isinstance(value, list):
                        # Combine multiple filters with OR
                        or_conditions = []
                        for v in value:
                            or_conditions.append(
                                getattr(self.db_model, field) == v
                            )

                        query = query.filter(or_(*or_conditions))
                    else:
                        # If it's not a list, apply a simple equality filter
                        query = query.filter(
                            getattr(self.db_model, field) == value
                        )
                else:
                    if isinstance(value, list):
                        or_conditions = []
                        for v in value:
                            or_conditions.append(
                                getattr(self.db_model, field).ilike(
                                    f"%{str(v)}%"
                                )
                            )

                        query = query.filter(or_(*or_conditions))
                    elif isinstance(value, bool):
                        if value is True:
                            # If true, the field has a value and the value
                            query = query.filter(
                                getattr(self.db_model, field).has()
                            )
                        else:
                            query = query.filter(
                                ~getattr(self.db_model, field).has()
                            )
                    elif isinstance(value, int):
                        query = query.filter(
                            getattr(self.db_model, field) == value
                        )
                    else:
                        # Apply a LIKE filter for string matching
                        query = query.filter(
                            func.coalesce(
                                getattr(self.db_model, field), ""
                            ).ilike(f"%{str(value)}%")
                        )

        count = await session.exec(query)
        total_count = count.one()

        if len(range) == 2:
            start, end = range
        else:
            start, end = [0, total_count]  # For content-range header

        response.headers["Content-Range"] = (
            f"sensor {start}-{end}/{total_count}"
        )

        return total_count

    async def get_model_by_id(
        self,
        session: AsyncSession,
        *,
        model_id: UUID,
    ) -> Any:
        """Get a model by id"""

        res = await session.exec(
            select(self.db_model).where(self.db_model.id == model_id)
        )
        obj = res.one_or_none()

        return obj

    async def create_model(
        self,
        model: Any,
        session: AsyncSession,
    ) -> Any:
        """Create a model"""

        obj = self.db_model.from_orm(model)

        session.add(obj)
        await session.commit()
        await session.refresh(obj)

        return obj
