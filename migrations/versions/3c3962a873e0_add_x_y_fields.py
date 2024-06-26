"""Add X Y fields

Revision ID: 3c3962a873e0
Revises: ce7ddc6e976a
Create Date: 2024-06-27 16:31:59.499158

"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa
import sqlmodel


# revision identifiers, used by Alembic.
revision: str = '3c3962a873e0'
down_revision: Union[str, None] = 'ce7ddc6e976a'
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # ### commands auto generated by Alembic - please adjust! ###
    op.add_column('gnss', sa.Column('x', sa.Float(), nullable=True))
    op.add_column('gnss', sa.Column('y', sa.Float(), nullable=True))
    op.drop_column('gnss', 'elevation_corrected')
    # ### end Alembic commands ###


def downgrade() -> None:
    # ### commands auto generated by Alembic - please adjust! ###
    op.add_column('gnss', sa.Column('elevation_corrected', sa.DOUBLE_PRECISION(precision=53), autoincrement=False, nullable=True))
    op.drop_column('gnss', 'y')
    op.drop_column('gnss', 'x')
    # ### end Alembic commands ###
