"""Add last_updated column

Revision ID: 53976e3abbd6
Revises: a97c8797c196
Create Date: 2024-07-11 16:18:52.957958

"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa
import sqlmodel


# revision identifiers, used by Alembic.
revision: str = '53976e3abbd6'
down_revision: Union[str, None] = 'a97c8797c196'
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # ### commands auto generated by Alembic - please adjust! ###
    op.add_column('instrumentexperiment', sa.Column('last_updated', sa.DateTime(), server_default=sa.text('now()'), nullable=False))
    # ### end Alembic commands ###


def downgrade() -> None:
    # ### commands auto generated by Alembic - please adjust! ###
    op.drop_column('instrumentexperiment', 'last_updated')
    # ### end Alembic commands ###
