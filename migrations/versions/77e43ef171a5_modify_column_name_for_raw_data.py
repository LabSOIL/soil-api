"""Modify column name for raw data

Revision ID: 77e43ef171a5
Revises: 9ec62d3cc28c
Create Date: 2024-07-12 15:31:01.493937

"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa
import sqlmodel
from sqlalchemy.dialects import postgresql

# revision identifiers, used by Alembic.
revision: str = '77e43ef171a5'
down_revision: Union[str, None] = '9ec62d3cc28c'
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # ### commands auto generated by Alembic - please adjust! ###
    op.add_column('instrumentexperimentchannel', sa.Column('baseline_values', sa.JSON(), nullable=True))
    op.drop_column('instrumentexperimentchannel', 'baseline_points')
    # ### end Alembic commands ###


def downgrade() -> None:
    # ### commands auto generated by Alembic - please adjust! ###
    op.add_column('instrumentexperimentchannel', sa.Column('baseline_points', postgresql.JSON(astext_type=sa.Text()), autoincrement=False, nullable=True))
    op.drop_column('instrumentexperimentchannel', 'baseline_values')
    # ### end Alembic commands ###