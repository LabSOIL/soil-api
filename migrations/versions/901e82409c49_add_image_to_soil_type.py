"""Add image to soil type

Revision ID: 901e82409c49
Revises: 128f1f044730
Create Date: 2024-06-26 10:46:22.547016

"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa
import sqlmodel


# revision identifiers, used by Alembic.
revision: str = '901e82409c49'
down_revision: Union[str, None] = '128f1f044730'
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # ### commands auto generated by Alembic - please adjust! ###
    op.add_column('soiltype', sa.Column('image', sqlmodel.sql.sqltypes.AutoString(), nullable=True))
    # ### end Alembic commands ###


def downgrade() -> None:
    # ### commands auto generated by Alembic - please adjust! ###
    op.drop_column('soiltype', 'image')
    # ### end Alembic commands ###
