"""Add relational field to plot tsvector

Revision ID: b1d731e141ba
Revises: 423836342799
Create Date: 2024-07-17 09:01:24.118050

"""

from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa
import sqlmodel
import sqlalchemy_utils


# revision identifiers, used by Alembic.
revision: str = "b1d731e141ba"
down_revision: Union[str, None] = "423836342799"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # ### commands auto generated by Alembic - please adjust! ###
    pass
    # ### end Alembic commands ###


def downgrade() -> None:
    # ### commands auto generated by Alembic - please adjust! ###
    pass
    # ### end Alembic commands ###
