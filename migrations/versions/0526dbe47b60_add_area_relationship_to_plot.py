"""Add area relationship to plot

Revision ID: 0526dbe47b60
Revises: 77f95220072c
Create Date: 2024-05-06 09:27:17.677080

"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa
import sqlmodel


# revision identifiers, used by Alembic.
revision: str = '0526dbe47b60'
down_revision: Union[str, None] = '77f95220072c'
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # ### commands auto generated by Alembic - please adjust! ###
    op.add_column('plot', sa.Column('area_id', sqlmodel.sql.sqltypes.GUID(), nullable=False))
    op.create_index(op.f('ix_plot_area_id'), 'plot', ['area_id'], unique=False)
    op.create_foreign_key(None, 'plot', 'area', ['area_id'], ['id'])
    # ### end Alembic commands ###


def downgrade() -> None:
    # ### commands auto generated by Alembic - please adjust! ###
    op.drop_constraint(None, 'plot', type_='foreignkey')
    op.drop_index(op.f('ix_plot_area_id'), table_name='plot')
    op.drop_column('plot', 'area_id')
    # ### end Alembic commands ###
