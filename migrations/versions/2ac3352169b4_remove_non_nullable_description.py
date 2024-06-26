"""Remove non-nullable description

Revision ID: 2ac3352169b4
Revises: 
Create Date: 2024-05-14 18:03:30.360002

"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa
import sqlmodel
from geoalchemy2 import Geometry

# revision identifiers, used by Alembic.
revision: str = '2ac3352169b4'
down_revision: Union[str, None] = None
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # ### commands auto generated by Alembic - please adjust! ###
    op.create_table('project',
    sa.Column('name', sqlmodel.sql.sqltypes.AutoString(), nullable=False),
    sa.Column('description', sqlmodel.sql.sqltypes.AutoString(), nullable=True),
    sa.Column('color', sqlmodel.sql.sqltypes.AutoString(), nullable=False),
    sa.Column('iterator', sa.Integer(), nullable=False),
    sa.Column('id', sqlmodel.sql.sqltypes.GUID(), nullable=False),
    sa.PrimaryKeyConstraint('iterator'),
    sa.UniqueConstraint('id'),
    sa.UniqueConstraint('name')
    )
    op.create_index(op.f('ix_project_id'), 'project', ['id'], unique=False)
    op.create_index(op.f('ix_project_iterator'), 'project', ['iterator'], unique=False)
    op.create_table('soiltype',
    sa.Column('name', sqlmodel.sql.sqltypes.AutoString(), nullable=False),
    sa.Column('description', sqlmodel.sql.sqltypes.AutoString(), nullable=False),
    sa.Column('iterator', sa.Integer(), nullable=False),
    sa.Column('id', sqlmodel.sql.sqltypes.GUID(), nullable=False),
    sa.PrimaryKeyConstraint('iterator'),
    sa.UniqueConstraint('id')
    )
    op.create_index(op.f('ix_soiltype_id'), 'soiltype', ['id'], unique=False)
    op.create_index(op.f('ix_soiltype_iterator'), 'soiltype', ['iterator'], unique=False)
    op.create_index(op.f('ix_soiltype_name'), 'soiltype', ['name'], unique=False)
    op.create_table('area',
    sa.Column('name', sqlmodel.sql.sqltypes.AutoString(), nullable=False),
    sa.Column('description', sqlmodel.sql.sqltypes.AutoString(), nullable=True),
    sa.Column('project_id', sqlmodel.sql.sqltypes.GUID(), nullable=False),
    sa.Column('iterator', sa.Integer(), nullable=False),
    sa.Column('id', sqlmodel.sql.sqltypes.GUID(), nullable=False),
    sa.ForeignKeyConstraint(['project_id'], ['project.id'], ),
    sa.PrimaryKeyConstraint('iterator'),
    sa.UniqueConstraint('id'),
    sa.UniqueConstraint('name', 'project_id', name='name_project_id')
    )
    op.create_index(op.f('ix_area_id'), 'area', ['id'], unique=False)
    op.create_index(op.f('ix_area_iterator'), 'area', ['iterator'], unique=False)
    op.create_index(op.f('ix_area_name'), 'area', ['name'], unique=False)
    op.create_index(op.f('ix_area_project_id'), 'area', ['project_id'], unique=False)
    op.create_geospatial_table('plot',
    sa.Column('name', sqlmodel.sql.sqltypes.AutoString(), nullable=False),
    sa.Column('plot_iterator', sa.Integer(), nullable=False),
    sa.Column('area_id', sqlmodel.sql.sqltypes.GUID(), nullable=False),
    sa.Column('gradient', sqlmodel.sql.sqltypes.AutoString(), nullable=False),
    sa.Column('vegetation_type', sqlmodel.sql.sqltypes.AutoString(), nullable=True),
    sa.Column('topography', sqlmodel.sql.sqltypes.AutoString(), nullable=True),
    sa.Column('aspect', sqlmodel.sql.sqltypes.AutoString(), nullable=True),
    sa.Column('created_on', sa.Date(), nullable=True),
    sa.Column('slope', sqlmodel.sql.sqltypes.AutoString(), nullable=True),
    sa.Column('weather', sqlmodel.sql.sqltypes.AutoString(), nullable=True),
    sa.Column('lithology', sqlmodel.sql.sqltypes.AutoString(), nullable=True),
    sa.Column('iterator', sa.Integer(), nullable=False),
    sa.Column('id', sqlmodel.sql.sqltypes.GUID(), nullable=False),
    sa.Column('geom', Geometry(geometry_type='POINTZ', srid=2056, spatial_index=False, from_text='ST_GeomFromEWKT', name='geometry'), nullable=True),
    sa.ForeignKeyConstraint(['area_id'], ['area.id'], ),
    sa.PrimaryKeyConstraint('iterator'),
    sa.UniqueConstraint('id'),
    sa.UniqueConstraint('name', name='unique_plot_name'),
    sa.UniqueConstraint('plot_iterator', 'area_id', 'gradient', name='unique_plot')
    )
    op.create_geospatial_index('idx_plot_geom', 'plot', ['geom'], unique=False, postgresql_using='gist', postgresql_ops={})
    op.create_index(op.f('ix_plot_area_id'), 'plot', ['area_id'], unique=False)
    op.create_index(op.f('ix_plot_gradient'), 'plot', ['gradient'], unique=False)
    op.create_index(op.f('ix_plot_id'), 'plot', ['id'], unique=False)
    op.create_index(op.f('ix_plot_iterator'), 'plot', ['iterator'], unique=False)
    op.create_index(op.f('ix_plot_name'), 'plot', ['name'], unique=False)
    op.create_index(op.f('ix_plot_plot_iterator'), 'plot', ['plot_iterator'], unique=False)
    op.create_geospatial_table('sensor',
    sa.Column('name', sqlmodel.sql.sqltypes.AutoString(), nullable=False),
    sa.Column('description', sqlmodel.sql.sqltypes.AutoString(), nullable=True),
    sa.Column('comment', sqlmodel.sql.sqltypes.AutoString(), nullable=True),
    sa.Column('elevation', sa.Float(), nullable=True),
    sa.Column('time_recorded_at_utc', sa.DateTime(), nullable=True),
    sa.Column('iterator', sa.Integer(), nullable=False),
    sa.Column('id', sqlmodel.sql.sqltypes.GUID(), nullable=False),
    sa.Column('time_ingested_at_utc', sa.DateTime(), nullable=False),
    sa.Column('geom', Geometry(geometry_type='POINTZ', srid=2056, spatial_index=False, from_text='ST_GeomFromEWKT', name='geometry'), nullable=True),
    sa.Column('area_id', sqlmodel.sql.sqltypes.GUID(), nullable=False),
    sa.ForeignKeyConstraint(['area_id'], ['area.id'], ),
    sa.PrimaryKeyConstraint('iterator'),
    sa.UniqueConstraint('id')
    )
    op.create_geospatial_index('idx_sensor_geom', 'sensor', ['geom'], unique=False, postgresql_using='gist', postgresql_ops={})
    op.create_index(op.f('ix_sensor_id'), 'sensor', ['id'], unique=False)
    op.create_index(op.f('ix_sensor_iterator'), 'sensor', ['iterator'], unique=False)
    op.create_index(op.f('ix_sensor_name'), 'sensor', ['name'], unique=False)
    op.create_index(op.f('ix_sensor_time_ingested_at_utc'), 'sensor', ['time_ingested_at_utc'], unique=False)
    op.create_index(op.f('ix_sensor_time_recorded_at_utc'), 'sensor', ['time_recorded_at_utc'], unique=False)
    op.create_geospatial_table('soilprofile',
    sa.Column('name', sqlmodel.sql.sqltypes.AutoString(), nullable=False),
    sa.Column('profile_iterator', sa.Integer(), nullable=False),
    sa.Column('gradient', sqlmodel.sql.sqltypes.AutoString(), nullable=False),
    sa.Column('description_horizon', sa.JSON(), nullable=True),
    sa.Column('weather', sqlmodel.sql.sqltypes.AutoString(), nullable=True),
    sa.Column('topography', sqlmodel.sql.sqltypes.AutoString(), nullable=True),
    sa.Column('vegetation_type', sqlmodel.sql.sqltypes.AutoString(), nullable=True),
    sa.Column('aspect', sqlmodel.sql.sqltypes.AutoString(), nullable=True),
    sa.Column('slope', sa.Float(), nullable=True),
    sa.Column('lythology_surficial_deposit', sqlmodel.sql.sqltypes.AutoString(), nullable=True),
    sa.Column('created_on', sa.DateTime(), nullable=True),
    sa.Column('soil_type_id', sqlmodel.sql.sqltypes.GUID(), nullable=False),
    sa.Column('area_id', sqlmodel.sql.sqltypes.GUID(), nullable=False),
    sa.Column('iterator', sa.Integer(), nullable=False),
    sa.Column('id', sqlmodel.sql.sqltypes.GUID(), nullable=False),
    sa.Column('geom', Geometry(geometry_type='POINTZ', srid=2056, spatial_index=False, from_text='ST_GeomFromEWKT', name='geometry'), nullable=True),
    sa.ForeignKeyConstraint(['area_id'], ['area.id'], ),
    sa.ForeignKeyConstraint(['soil_type_id'], ['soiltype.id'], ),
    sa.PrimaryKeyConstraint('iterator'),
    sa.UniqueConstraint('id'),
    sa.UniqueConstraint('profile_iterator', 'area_id', 'gradient', name='unique_profile')
    )
    op.create_geospatial_index('idx_soilprofile_geom', 'soilprofile', ['geom'], unique=False, postgresql_using='gist', postgresql_ops={})
    op.create_index(op.f('ix_soilprofile_area_id'), 'soilprofile', ['area_id'], unique=False)
    op.create_index(op.f('ix_soilprofile_created_on'), 'soilprofile', ['created_on'], unique=False)
    op.create_index(op.f('ix_soilprofile_gradient'), 'soilprofile', ['gradient'], unique=False)
    op.create_index(op.f('ix_soilprofile_id'), 'soilprofile', ['id'], unique=False)
    op.create_index(op.f('ix_soilprofile_iterator'), 'soilprofile', ['iterator'], unique=False)
    op.create_index(op.f('ix_soilprofile_name'), 'soilprofile', ['name'], unique=False)
    op.create_index(op.f('ix_soilprofile_profile_iterator'), 'soilprofile', ['profile_iterator'], unique=False)
    op.create_index(op.f('ix_soilprofile_soil_type_id'), 'soilprofile', ['soil_type_id'], unique=False)
    op.create_table('plotsample',
    sa.Column('name', sa.Enum('A', 'B', 'C', name='plotsamplenames'), nullable=False),
    sa.Column('upper_depth_cm', sa.Float(), nullable=False),
    sa.Column('lower_depth_cm', sa.Float(), nullable=False),
    sa.Column('plot_id', sqlmodel.sql.sqltypes.GUID(), nullable=False),
    sa.Column('sample_weight', sa.Float(), nullable=False),
    sa.Column('subsample_weight', sa.Float(), nullable=True),
    sa.Column('ph', sa.Float(), nullable=True),
    sa.Column('rh', sa.Float(), nullable=True),
    sa.Column('loi', sa.Float(), nullable=True),
    sa.Column('mfc', sa.Float(), nullable=True),
    sa.Column('c', sa.Float(), nullable=True),
    sa.Column('n', sa.Float(), nullable=True),
    sa.Column('cn', sa.Float(), nullable=True),
    sa.Column('clay_percent', sa.Float(), nullable=True),
    sa.Column('silt_percent', sa.Float(), nullable=True),
    sa.Column('sand_percent', sa.Float(), nullable=True),
    sa.Column('fe_ug_per_g', sa.Float(), nullable=True),
    sa.Column('na_ug_per_g', sa.Float(), nullable=True),
    sa.Column('al_ug_per_g', sa.Float(), nullable=True),
    sa.Column('k_ug_per_g', sa.Float(), nullable=True),
    sa.Column('ca_ug_per_g', sa.Float(), nullable=True),
    sa.Column('mg_ug_per_g', sa.Float(), nullable=True),
    sa.Column('mn_ug_per_g', sa.Float(), nullable=True),
    sa.Column('s_ug_per_g', sa.Float(), nullable=True),
    sa.Column('cl_ug_per_g', sa.Float(), nullable=True),
    sa.Column('p_ug_per_g', sa.Float(), nullable=True),
    sa.Column('si_ug_per_g', sa.Float(), nullable=True),
    sa.Column('iterator', sa.Integer(), nullable=False),
    sa.Column('id', sqlmodel.sql.sqltypes.GUID(), nullable=False),
    sa.ForeignKeyConstraint(['plot_id'], ['plot.id'], ),
    sa.PrimaryKeyConstraint('iterator'),
    sa.UniqueConstraint('id'),
    sa.UniqueConstraint('name', 'plot_id', name='unique_plot_sample')
    )
    op.create_index(op.f('ix_plotsample_id'), 'plotsample', ['id'], unique=False)
    op.create_index(op.f('ix_plotsample_iterator'), 'plotsample', ['iterator'], unique=False)
    op.create_index(op.f('ix_plotsample_name'), 'plotsample', ['name'], unique=False)
    op.create_index(op.f('ix_plotsample_plot_id'), 'plotsample', ['plot_id'], unique=False)
    op.create_table('sensordata',
    sa.Column('instrument_seq', sa.Integer(), nullable=False),
    sa.Column('time', sa.DateTime(), nullable=False),
    sa.Column('time_zone', sa.Integer(), nullable=True),
    sa.Column('temperature_1', sa.Float(), nullable=True),
    sa.Column('temperature_2', sa.Float(), nullable=True),
    sa.Column('temperature_3', sa.Float(), nullable=True),
    sa.Column('soil_moisture_count', sa.Float(), nullable=True),
    sa.Column('shake', sa.Integer(), nullable=True),
    sa.Column('error_flat', sa.Integer(), nullable=True),
    sa.Column('iterator', sa.Integer(), nullable=False),
    sa.Column('id', sqlmodel.sql.sqltypes.GUID(), nullable=False),
    sa.Column('sensor_id', sqlmodel.sql.sqltypes.GUID(), nullable=False),
    sa.ForeignKeyConstraint(['sensor_id'], ['sensor.id'], ),
    sa.PrimaryKeyConstraint('iterator'),
    sa.UniqueConstraint('id')
    )
    op.create_index(op.f('ix_sensordata_id'), 'sensordata', ['id'], unique=False)
    op.create_index(op.f('ix_sensordata_instrument_seq'), 'sensordata', ['instrument_seq'], unique=False)
    op.create_index(op.f('ix_sensordata_iterator'), 'sensordata', ['iterator'], unique=False)
    op.create_index(op.f('ix_sensordata_sensor_id'), 'sensordata', ['sensor_id'], unique=False)
    op.create_index(op.f('ix_sensordata_soil_moisture_count'), 'sensordata', ['soil_moisture_count'], unique=False)
    op.create_index(op.f('ix_sensordata_temperature_1'), 'sensordata', ['temperature_1'], unique=False)
    op.create_index(op.f('ix_sensordata_temperature_2'), 'sensordata', ['temperature_2'], unique=False)
    op.create_index(op.f('ix_sensordata_temperature_3'), 'sensordata', ['temperature_3'], unique=False)
    op.create_index(op.f('ix_sensordata_time'), 'sensordata', ['time'], unique=False)
    # ### end Alembic commands ###


def downgrade() -> None:
    # ### commands auto generated by Alembic - please adjust! ###
    op.drop_index(op.f('ix_sensordata_time'), table_name='sensordata')
    op.drop_index(op.f('ix_sensordata_temperature_3'), table_name='sensordata')
    op.drop_index(op.f('ix_sensordata_temperature_2'), table_name='sensordata')
    op.drop_index(op.f('ix_sensordata_temperature_1'), table_name='sensordata')
    op.drop_index(op.f('ix_sensordata_soil_moisture_count'), table_name='sensordata')
    op.drop_index(op.f('ix_sensordata_sensor_id'), table_name='sensordata')
    op.drop_index(op.f('ix_sensordata_iterator'), table_name='sensordata')
    op.drop_index(op.f('ix_sensordata_instrument_seq'), table_name='sensordata')
    op.drop_index(op.f('ix_sensordata_id'), table_name='sensordata')
    op.drop_table('sensordata')
    op.drop_index(op.f('ix_plotsample_plot_id'), table_name='plotsample')
    op.drop_index(op.f('ix_plotsample_name'), table_name='plotsample')
    op.drop_index(op.f('ix_plotsample_iterator'), table_name='plotsample')
    op.drop_index(op.f('ix_plotsample_id'), table_name='plotsample')
    op.drop_table('plotsample')
    op.drop_index(op.f('ix_soilprofile_soil_type_id'), table_name='soilprofile')
    op.drop_index(op.f('ix_soilprofile_profile_iterator'), table_name='soilprofile')
    op.drop_index(op.f('ix_soilprofile_name'), table_name='soilprofile')
    op.drop_index(op.f('ix_soilprofile_iterator'), table_name='soilprofile')
    op.drop_index(op.f('ix_soilprofile_id'), table_name='soilprofile')
    op.drop_index(op.f('ix_soilprofile_gradient'), table_name='soilprofile')
    op.drop_index(op.f('ix_soilprofile_created_on'), table_name='soilprofile')
    op.drop_index(op.f('ix_soilprofile_area_id'), table_name='soilprofile')
    op.drop_geospatial_index('idx_soilprofile_geom', table_name='soilprofile', postgresql_using='gist', column_name='geom')
    op.drop_geospatial_table('soilprofile')
    op.drop_index(op.f('ix_sensor_time_recorded_at_utc'), table_name='sensor')
    op.drop_index(op.f('ix_sensor_time_ingested_at_utc'), table_name='sensor')
    op.drop_index(op.f('ix_sensor_name'), table_name='sensor')
    op.drop_index(op.f('ix_sensor_iterator'), table_name='sensor')
    op.drop_index(op.f('ix_sensor_id'), table_name='sensor')
    op.drop_geospatial_index('idx_sensor_geom', table_name='sensor', postgresql_using='gist', column_name='geom')
    op.drop_geospatial_table('sensor')
    op.drop_index(op.f('ix_plot_plot_iterator'), table_name='plot')
    op.drop_index(op.f('ix_plot_name'), table_name='plot')
    op.drop_index(op.f('ix_plot_iterator'), table_name='plot')
    op.drop_index(op.f('ix_plot_id'), table_name='plot')
    op.drop_index(op.f('ix_plot_gradient'), table_name='plot')
    op.drop_index(op.f('ix_plot_area_id'), table_name='plot')
    op.drop_geospatial_index('idx_plot_geom', table_name='plot', postgresql_using='gist', column_name='geom')
    op.drop_geospatial_table('plot')
    op.drop_index(op.f('ix_area_project_id'), table_name='area')
    op.drop_index(op.f('ix_area_name'), table_name='area')
    op.drop_index(op.f('ix_area_iterator'), table_name='area')
    op.drop_index(op.f('ix_area_id'), table_name='area')
    op.drop_table('area')
    op.drop_index(op.f('ix_soiltype_name'), table_name='soiltype')
    op.drop_index(op.f('ix_soiltype_iterator'), table_name='soiltype')
    op.drop_index(op.f('ix_soiltype_id'), table_name='soiltype')
    op.drop_table('soiltype')
    op.drop_index(op.f('ix_project_iterator'), table_name='project')
    op.drop_index(op.f('ix_project_id'), table_name='project')
    op.drop_table('project')
    # ### end Alembic commands ###
