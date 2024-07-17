from app.instruments.models.experiment import (
    InstrumentExperiment,
    InstrumentExperimentRead,
    InstrumentExperimentCreate,
    InstrumentExperimentUpdate,
)
from app.db import get_session, AsyncSession
from fastapi import Depends, APIRouter, Query, Response, HTTPException
from sqlmodel import select
from uuid import UUID
from typing import Any
from app.crud import CRUD
from app.plots.models import Plot
from app.instruments.services import (
    get_count,
    get_data,
    get_one,
    create_one,
    delete_one,
    delete_many,
    update_one,
)
import csv

router = APIRouter()


@router.get("/{id}", response_model=InstrumentExperimentRead)
async def get_instrument_experiment(
    obj: InstrumentExperiment = Depends(get_one),
) -> InstrumentExperimentRead:
    """Get an experiment by id"""

    return obj


@router.get("/{id}/raw")
async def get_instrument_experiment_rawdata(
    obj: InstrumentExperiment = Depends(get_one),
) -> Any:
    """Get an experiment's raw data by id, and all of its channels as CSV

    The time column is equivalent for each channel, the channel header is
    `channel_name` of each channel, and the value to fill is `raw_values`
    """

    header = ["Time/s"]
    # Sort channels by channel_name
    channels = sorted(obj.channels, key=lambda x: x.channel_name)
    header += [f"{channel.channel_name}" for channel in channels]

    # Form CSV by looping through each column and its data using the structure
    # defined in the docstring
    csv_data = [header]
    for i in range(len(obj.channels[0].raw_values)):
        row = [obj.channels[0].time_values[i]]
        row += [channel.raw_values[i] for channel in channels]
        csv_data.append(row)

    return csv_data


@router.get("/{id}/filtered")
async def get_instrument_experiment_baseline_filtered_data(
    obj: InstrumentExperiment = Depends(get_one),
) -> Any:
    """Get an experiment's baseline filtered data as CSV

    The time column is equivalent for each channel, the channel header is
    `channel_name` of each channel, and the value to fill is `baseline_values`
    """

    header = ["Time/s"]
    # Sort channels by channel_name
    channels = sorted(obj.channels, key=lambda x: x.channel_name)
    header += [f"{channel.channel_name}" for channel in channels]

    # Form CSV by looping through each column and its data using the structure
    # defined in the docstring
    csv_data = [header]
    for i in range(len(obj.channels[0].raw_values)):
        row = [obj.channels[0].time_values[i]]

        # If there are no baseline values for a channel at the current index
        # then fill with None
        for channel in channels:
            if len(channel.baseline_values) <= i:
                row.append(None)
            else:
                row.append(channel.baseline_values[i])

        # row += [channel.baseline_values[i] for channel in channels]
        csv_data.append(row)

    return csv_data


@router.get("/{id}/summary")
async def get_instrument_experiment_summary_data(
    obj: InstrumentExperiment = Depends(get_one),
) -> Any:
    """Create a CSV return that returns the channel integral data"""

    header = ["measurement"]
    channels = sorted(obj.channels, key=lambda x: x.channel_name)

    # Find the maximum number of samples
    max_samples = max(len(channel.integral_results) for channel in channels)

    # Construct the header
    for i in range(1, max_samples + 1):
        header += [
            f"sample{i}_start",
            f"sample{i}_end",
            f"sample{i}_area",
        ]

    # Create CSV rows
    csv_data = [header]
    for channel in channels:
        row = [channel.channel_name]
        for sample in channel.integral_results:
            row += [
                sample.get("start", "nan"),
                sample.get("end", "nan"),
                sample.get("area", "nan"),
            ]
        # Fill remaining values with 'nan' if the channel has fewer samples
        remaining_samples = max_samples - len(channel.integral_results)
        row += ["nan"] * (
            remaining_samples * 3
        )  # Adjust multiplier if peak_time and peak_value are included
        csv_data.append(row)

    return csv_data


@router.get("", response_model=list[InstrumentExperimentRead])
async def get_all_instrument_experiments(
    response: Response,
    obj: CRUD = Depends(get_data),
    total_count: int = Depends(get_count),
) -> list[InstrumentExperimentRead]:
    """Get all InstrumentExperiment data"""

    return obj


@router.post("", response_model=InstrumentExperimentRead)
async def create_instrument_experiment(
    obj: InstrumentExperiment = Depends(create_one),
) -> InstrumentExperimentRead:
    """Creates a instrument_experiment data record"""

    return obj


@router.put("/{id}", response_model=InstrumentExperimentRead)
async def update_instrument_experiment(
    obj: InstrumentExperiment = Depends(update_one),
) -> InstrumentExperimentRead:
    """Update a instrument_experiment by id"""

    return obj


@router.delete("/batch", response_model=list[UUID])
async def delete_batch(
    deleted_ids: list[UUID] = Depends(delete_many),
) -> list[UUID]:
    """Delete by a list of ids"""

    return deleted_ids


@router.delete("/{id}", response_model=UUID)
async def delete_instrument_experiment(
    deleted_id: UUID = Depends(delete_one),
) -> UUID:
    """Delete a instrument_experiment by id"""

    return deleted_id
