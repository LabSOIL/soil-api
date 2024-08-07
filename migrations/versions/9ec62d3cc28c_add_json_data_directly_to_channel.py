"""Add JSON data directly to Channel

Revision ID: 9ec62d3cc28c
Revises: c5245bffc72f
Create Date: 2024-07-12 15:22:30.717399

"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa
import sqlmodel


# revision identifiers, used by Alembic.
revision: str = '9ec62d3cc28c'
down_revision: Union[str, None] = 'c5245bffc72f'
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # ### commands auto generated by Alembic - please adjust! ###
    op.add_column('instrumentexperimentchannel', sa.Column('time_values', sa.JSON(), nullable=True))
    op.add_column('instrumentexperimentchannel', sa.Column('raw_values', sa.JSON(), nullable=True))
    # ### end Alembic commands ###


def downgrade() -> None:
    # ### commands auto generated by Alembic - please adjust! ###
    op.drop_column('instrumentexperimentchannel', 'raw_values')
    op.drop_column('instrumentexperimentchannel', 'time_values')
    # ### end Alembic commands ###
