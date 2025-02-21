use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // PostGIS specific
        let create_all_tables = r#"
            CREATE EXTENSION IF NOT EXISTS postgis;
            -- ===============================================
            -- Migration Script: Create Brand-New Database Schema
            -- with indexes on all text, numeric and date fields
            -- ===============================================

            -------------------------------
            -- 1. Create required ENUM types
            -------------------------------
            CREATE TYPE public.gradientchoices AS ENUM ('flat', 'slope');

            -------------------------------
            -- 2. Create independent tables
            -------------------------------

            -- 2a. PROJECT
            CREATE TABLE public.project (
                id uuid NOT NULL,
                name character varying NOT NULL,
                description character varying,
                color character varying NOT NULL,
                last_updated timestamp without time zone DEFAULT now() NOT NULL,
                CONSTRAINT project_pkey PRIMARY KEY (id),
                CONSTRAINT project_name_key UNIQUE (name)
            );

            -- 2b. SOILTYPE
            CREATE TABLE public.soiltype (
                id uuid NOT NULL,
                name character varying NOT NULL,
                description character varying NOT NULL,
                last_updated timestamp without time zone DEFAULT now() NOT NULL,
                image character varying,
                CONSTRAINT soiltype_pkey PRIMARY KEY (id)
            );

            -------------------------------
            -- 3. Create tables that depend on the above
            -------------------------------

            -- 3a. AREA (depends on project)
            CREATE TABLE public.area (
                id uuid NOT NULL,
                name character varying NOT NULL,
                description character varying,
                project_id uuid NOT NULL,
                last_updated timestamp without time zone DEFAULT now() NOT NULL,
                CONSTRAINT area_pkey PRIMARY KEY (id),
                CONSTRAINT name_project_id UNIQUE (name, project_id),
                CONSTRAINT area_project_id_fkey FOREIGN KEY (project_id) REFERENCES public.project(id)
            );

            -- 3b. GNSS (no FK dependencies)
            CREATE TABLE public.gnss (
                id uuid NOT NULL,
                name character varying,
                last_updated timestamp without time zone DEFAULT now() NOT NULL,
                latitude double precision,
                longitude double precision,
                "time" timestamp without time zone,
                comment character varying,
                original_filename character varying,
                elevation_gps double precision,
                coord_x double precision GENERATED ALWAYS AS (
                    public.st_x(public.st_transform(public.st_setsrid(public.st_makepoint(longitude, latitude), 4326), 2056))
                ) STORED,
                coord_y double precision GENERATED ALWAYS AS (
                    public.st_y(public.st_transform(public.st_setsrid(public.st_makepoint(longitude, latitude), 4326), 2056))
                ) STORED,
                coord_srid integer GENERATED ALWAYS AS (2056) STORED,
                geom public.geometry(PointZ,2056) GENERATED ALWAYS AS (
                    public.st_transform(public.st_setsrid(public.st_makepoint(longitude, latitude, elevation_gps), 4326), 2056)
                ) STORED,
                CONSTRAINT gnss_pkey PRIMARY KEY (id)
            );

            -- 3c. INSTRUMENTEXPERIMENT (depends on project)
            CREATE TABLE public.instrumentexperiment (
                id uuid NOT NULL,
                name character varying,
                date timestamp without time zone,
                description character varying,
                filename character varying,
                device_filename character varying,
                data_source character varying,
                instrument_model character varying,
                init_e double precision,
                sample_interval double precision,
                run_time double precision,
                quiet_time double precision,
                sensitivity double precision,
                samples integer,
                last_updated timestamp without time zone DEFAULT now() NOT NULL,
                project_id uuid,
                CONSTRAINT instrumentexperiment_pkey PRIMARY KEY (id),
                CONSTRAINT instrumentexperiment_project_id_fkey FOREIGN KEY (project_id) REFERENCES public.project(id)
            );

            -- 3d. INSTRUMENTEXPERIMENTCHANNEL (depends on instrumentexperiment)
            CREATE TABLE public.instrumentexperimentchannel (
                id uuid NOT NULL,
                channel_name character varying NOT NULL,
                experiment_id uuid NOT NULL,
                baseline_spline json,
                time_values json,
                raw_values json,
                baseline_values json,
                baseline_chosen_points json,
                integral_chosen_pairs json,
                integral_results json,
                CONSTRAINT instrumentexperimentchannel_pkey PRIMARY KEY (id),
                CONSTRAINT instrumentexperimentchannel_experiment_id_fkey FOREIGN KEY (experiment_id)
                    REFERENCES public.instrumentexperiment(id)
            );

            -- 3e. PLOT (depends on area and uses the gradient enum)
            CREATE TABLE public.plot (
                id uuid NOT NULL,
                name character varying NOT NULL,
                area_id uuid NOT NULL,
                gradient public.gradientchoices NOT NULL,
                vegetation_type character varying,
                topography character varying,
                aspect character varying,
                created_on date,
                weather character varying,
                lithology character varying,
                last_updated timestamp without time zone DEFAULT now() NOT NULL,
                image character varying,
                coord_x double precision,
                coord_y double precision,
                coord_z double precision,
                coord_srid integer,
                geom public.geometry(PointZ) GENERATED ALWAYS AS (
                    public.st_setsrid(public.st_makepoint(coord_x, coord_y, coord_z), coord_srid)
                ) STORED,
                CONSTRAINT plot_pkey PRIMARY KEY (id),
                CONSTRAINT unique_plot_name UNIQUE (name),
                CONSTRAINT plot_area_id_fkey FOREIGN KEY (area_id) REFERENCES public.area(id)
            );

            -- 3f. PLOTSAMPLE (depends on plot)
            CREATE TABLE public.plotsample (
                id uuid NOT NULL,
                name character varying NOT NULL,
                upper_depth_cm double precision NOT NULL,
                lower_depth_cm double precision NOT NULL,
                plot_id uuid NOT NULL,
                sample_weight double precision,
                subsample_weight double precision,
                ph double precision,
                rh double precision,
                loi double precision,
                mfc double precision,
                c double precision,
                n double precision,
                cn double precision,
                clay_percent double precision,
                silt_percent double precision,
                sand_percent double precision,
                fe_ug_per_g double precision,
                na_ug_per_g double precision,
                al_ug_per_g double precision,
                k_ug_per_g double precision,
                ca_ug_per_g double precision,
                mg_ug_per_g double precision,
                mn_ug_per_g double precision,
                s_ug_per_g double precision,
                cl_ug_per_g double precision,
                p_ug_per_g double precision,
                si_ug_per_g double precision,
                subsample_replica_weight double precision,
                created_on date,
                last_updated timestamp without time zone DEFAULT now() NOT NULL,
                fungi_per_g double precision,
                bacteria_per_g double precision,
                archea_per_g double precision,
                methanogens_per_g double precision,
                methanotrophs_per_g double precision,
                replicate integer DEFAULT 1 NOT NULL,
                CONSTRAINT plotsample_pkey PRIMARY KEY (id),
                CONSTRAINT unique_plot_sample UNIQUE (name, plot_id),
                CONSTRAINT unique_plot_sample_depth UNIQUE (plot_id, replicate, upper_depth_cm, lower_depth_cm),
                CONSTRAINT plotsample_plot_id_fkey FOREIGN KEY (plot_id) REFERENCES public.plot(id)
            );

            -- 3g. TRANSECT (depends on area)
            CREATE TABLE public.transect (
                id uuid NOT NULL,
                name character varying,
                description character varying,
                date_created timestamp without time zone,
                last_updated timestamp without time zone DEFAULT now() NOT NULL,
                area_id uuid NOT NULL,
                CONSTRAINT transect_pkey PRIMARY KEY (id),
                CONSTRAINT transect_area_id_fkey FOREIGN KEY (area_id) REFERENCES public.area(id)
            );

            -- 3h. TRANSECTNODE (depends on transect and plot)
            CREATE TABLE public.transectnode (
                id uuid NOT NULL,
                plot_id uuid NOT NULL,
                transect_id uuid NOT NULL,
                "order" integer NOT NULL,
                CONSTRAINT transectnode_pkey PRIMARY KEY (id),
                CONSTRAINT no_same_link_constraint UNIQUE (transect_id, plot_id),
                CONSTRAINT no_same_order_constraint UNIQUE ("order", transect_id),
                CONSTRAINT transectnode_plot_id_fkey FOREIGN KEY (plot_id) REFERENCES public.plot(id),
                CONSTRAINT transectnode_transect_id_fkey FOREIGN KEY (transect_id) REFERENCES public.transect(id)
            );

            -- 3i. SENSOR (depends on area)
            CREATE TABLE public.sensor (
                id uuid NOT NULL,
                name character varying,
                description character varying,
                comment character varying,
                geom public.geometry(PointZ,2056),
                area_id uuid NOT NULL,
                last_updated timestamp without time zone DEFAULT now() NOT NULL,
                serial_number character varying,
                manufacturer character varying,
                CONSTRAINT sensor_pkey PRIMARY KEY (id),
                CONSTRAINT sensor_area_id_fkey FOREIGN KEY (area_id) REFERENCES public.area(id)
            );

            -- 3j. PLOTSENSORASSIGNMENTS (depends on plot and sensor)
            CREATE TABLE public.plotsensorassignments (
                id uuid NOT NULL,
                date_from timestamp without time zone NOT NULL,
                date_to timestamp without time zone NOT NULL,
                plot_id uuid NOT NULL,
                sensor_id uuid NOT NULL,
                depth_cm integer NOT NULL,
                CONSTRAINT plotsensorassignments_pkey PRIMARY KEY (id),
                CONSTRAINT plotsensorassignments_plot_id_fkey FOREIGN KEY (plot_id) REFERENCES public.plot(id),
                CONSTRAINT plotsensorassignments_sensor_id_fkey FOREIGN KEY (sensor_id) REFERENCES public.sensor(id)
            );

            -- 3k. SENSORDATA (depends on sensor)
            CREATE TABLE public.sensordata (
                id uuid NOT NULL,
                instrument_seq integer NOT NULL,
                time_zone integer,
                temperature_1 double precision,
                temperature_2 double precision,
                temperature_3 double precision,
                soil_moisture_count double precision,
                shake integer,
                error_flat integer,
                sensor_id uuid NOT NULL,
                last_updated timestamp without time zone NOT NULL,
                time_utc timestamp without time zone NOT NULL,
                temperature_average double precision,
                CONSTRAINT sensordata_pkey PRIMARY KEY (id),
                CONSTRAINT sensordata_instrument_seq_sensor_id_key UNIQUE (instrument_seq, sensor_id),
                CONSTRAINT sensordata_time_utc_sensor_id_key UNIQUE (time_utc, sensor_id),
                CONSTRAINT sensordata_sensor_id_fkey FOREIGN KEY (sensor_id) REFERENCES public.sensor(id)
            );

            -- 3l. SOILPROFILE (depends on area and soiltype)
            CREATE TABLE public.soilprofile (
                id uuid NOT NULL,
                name character varying NOT NULL,
                gradient character varying NOT NULL,
                description_horizon json,
                weather character varying,
                topography character varying,
                vegetation_type character varying,
                aspect character varying,
                lythology_surficial_deposit character varying,
                created_on timestamp without time zone,
                soil_type_id uuid NOT NULL,
                area_id uuid NOT NULL,
                last_updated timestamp without time zone DEFAULT now() NOT NULL,
                soil_diagram character varying,
                photo character varying,
                parent_material double precision,
                coord_x double precision,
                coord_y double precision,
                coord_z double precision,
                coord_srid integer,
                geom public.geometry(PointZ) GENERATED ALWAYS AS (
                    public.st_setsrid(public.st_makepoint(coord_x, coord_y, coord_z), coord_srid)
                ) STORED,
                CONSTRAINT soilprofile_pkey PRIMARY KEY (id),
                CONSTRAINT soilprofile_area_id_fkey FOREIGN KEY (area_id) REFERENCES public.area(id),
                CONSTRAINT soilprofile_soil_type_id_fkey FOREIGN KEY (soil_type_id) REFERENCES public.soiltype(id)
            );

            -------------------------------
            -- 4. Create spatial indexes
            -------------------------------
            CREATE INDEX IF NOT EXISTS idx_gnss_geom ON public.gnss USING gist (geom);
            CREATE INDEX IF NOT EXISTS idx_plot_geom ON public.plot USING gist (geom);
            CREATE INDEX IF NOT EXISTS idx_soilprofile_geom ON public.soilprofile USING gist (geom);
            CREATE INDEX IF NOT EXISTS idx_sensor_geom ON public.sensor USING gist (geom);

            -------------------------------
            -- 5. Create additional indexes on text, numeric, and date fields
            -------------------------------

            -- PROJECT table indexes
            CREATE INDEX IF NOT EXISTS idx_project_description ON public.project(description);
            CREATE INDEX IF NOT EXISTS idx_project_color ON public.project(color);
            CREATE INDEX IF NOT EXISTS idx_project_last_updated ON public.project(last_updated);

            -- SOILTYPE table indexes
            CREATE INDEX IF NOT EXISTS idx_soiltype_name ON public.soiltype(name);
            CREATE INDEX IF NOT EXISTS idx_soiltype_description ON public.soiltype(description);
            CREATE INDEX IF NOT EXISTS idx_soiltype_last_updated ON public.soiltype(last_updated);
            CREATE INDEX IF NOT EXISTS idx_soiltype_image ON public.soiltype(image);

            -- AREA table indexes
            CREATE INDEX IF NOT EXISTS idx_area_name ON public.area(name);
            CREATE INDEX IF NOT EXISTS idx_area_description ON public.area(description);
            CREATE INDEX IF NOT EXISTS idx_area_last_updated ON public.area(last_updated);

            -- GNSS table indexes
            CREATE INDEX IF NOT EXISTS idx_gnss_name ON public.gnss(name);
            CREATE INDEX IF NOT EXISTS idx_gnss_last_updated ON public.gnss(last_updated);
            CREATE INDEX IF NOT EXISTS idx_gnss_latitude ON public.gnss(latitude);
            CREATE INDEX IF NOT EXISTS idx_gnss_longitude ON public.gnss(longitude);
            CREATE INDEX IF NOT EXISTS idx_gnss_time ON public.gnss("time");
            CREATE INDEX IF NOT EXISTS idx_gnss_comment ON public.gnss(comment);
            CREATE INDEX IF NOT EXISTS idx_gnss_original_filename ON public.gnss(original_filename);
            CREATE INDEX IF NOT EXISTS idx_gnss_elevation_gps ON public.gnss(elevation_gps);
            CREATE INDEX IF NOT EXISTS idx_gnss_coord_x ON public.gnss(coord_x);
            CREATE INDEX IF NOT EXISTS idx_gnss_coord_y ON public.gnss(coord_y);

            -- INSTRUMENTEXPERIMENT table indexes
            CREATE INDEX IF NOT EXISTS idx_instrumentexperiment_name ON public.instrumentexperiment(name);
            CREATE INDEX IF NOT EXISTS idx_instrumentexperiment_date ON public.instrumentexperiment(date);
            CREATE INDEX IF NOT EXISTS idx_instrumentexperiment_description ON public.instrumentexperiment(description);
            CREATE INDEX IF NOT EXISTS idx_instrumentexperiment_filename ON public.instrumentexperiment(filename);
            CREATE INDEX IF NOT EXISTS idx_instrumentexperiment_device_filename ON public.instrumentexperiment(device_filename);
            CREATE INDEX IF NOT EXISTS idx_instrumentexperiment_data_source ON public.instrumentexperiment(data_source);
            CREATE INDEX IF NOT EXISTS idx_instrumentexperiment_instrument_model ON public.instrumentexperiment(instrument_model);
            CREATE INDEX IF NOT EXISTS idx_instrumentexperiment_init_e ON public.instrumentexperiment(init_e);
            CREATE INDEX IF NOT EXISTS idx_instrumentexperiment_sample_interval ON public.instrumentexperiment(sample_interval);
            CREATE INDEX IF NOT EXISTS idx_instrumentexperiment_run_time ON public.instrumentexperiment(run_time);
            CREATE INDEX IF NOT EXISTS idx_instrumentexperiment_quiet_time ON public.instrumentexperiment(quiet_time);
            CREATE INDEX IF NOT EXISTS idx_instrumentexperiment_sensitivity ON public.instrumentexperiment(sensitivity);
            CREATE INDEX IF NOT EXISTS idx_instrumentexperiment_samples ON public.instrumentexperiment(samples);
            CREATE INDEX IF NOT EXISTS idx_instrumentexperiment_last_updated ON public.instrumentexperiment(last_updated);

            -- INSTRUMENTEXPERIMENTCHANNEL table indexes
            CREATE INDEX IF NOT EXISTS idx_instrumentexperimentchannel_channel_name ON public.instrumentexperimentchannel(channel_name);

            -- PLOT table indexes
            CREATE INDEX IF NOT EXISTS idx_plot_gradient ON public.plot(gradient);
            CREATE INDEX IF NOT EXISTS idx_plot_vegetation_type ON public.plot(vegetation_type);
            CREATE INDEX IF NOT EXISTS idx_plot_topography ON public.plot(topography);
            CREATE INDEX IF NOT EXISTS idx_plot_aspect ON public.plot(aspect);
            CREATE INDEX IF NOT EXISTS idx_plot_created_on ON public.plot(created_on);
            CREATE INDEX IF NOT EXISTS idx_plot_weather ON public.plot(weather);
            CREATE INDEX IF NOT EXISTS idx_plot_lithology ON public.plot(lithology);
            CREATE INDEX IF NOT EXISTS idx_plot_last_updated ON public.plot(last_updated);
            CREATE INDEX IF NOT EXISTS idx_plot_image ON public.plot(image);
            CREATE INDEX IF NOT EXISTS idx_plot_coord_x ON public.plot(coord_x);
            CREATE INDEX IF NOT EXISTS idx_plot_coord_y ON public.plot(coord_y);
            CREATE INDEX IF NOT EXISTS idx_plot_coord_z ON public.plot(coord_z);

            -- PLOTSAMPLE table indexes
            CREATE INDEX IF NOT EXISTS idx_plotsample_name ON public.plotsample(name);
            CREATE INDEX IF NOT EXISTS idx_plotsample_upper_depth_cm ON public.plotsample(upper_depth_cm);
            CREATE INDEX IF NOT EXISTS idx_plotsample_lower_depth_cm ON public.plotsample(lower_depth_cm);
            CREATE INDEX IF NOT EXISTS idx_plotsample_sample_weight ON public.plotsample(sample_weight);
            CREATE INDEX IF NOT EXISTS idx_plotsample_subsample_weight ON public.plotsample(subsample_weight);
            CREATE INDEX IF NOT EXISTS idx_plotsample_ph ON public.plotsample(ph);
            CREATE INDEX IF NOT EXISTS idx_plotsample_rh ON public.plotsample(rh);
            CREATE INDEX IF NOT EXISTS idx_plotsample_loi ON public.plotsample(loi);
            CREATE INDEX IF NOT EXISTS idx_plotsample_mfc ON public.plotsample(mfc);
            CREATE INDEX IF NOT EXISTS idx_plotsample_c ON public.plotsample(c);
            CREATE INDEX IF NOT EXISTS idx_plotsample_n ON public.plotsample(n);
            CREATE INDEX IF NOT EXISTS idx_plotsample_cn ON public.plotsample(cn);
            CREATE INDEX IF NOT EXISTS idx_plotsample_clay_percent ON public.plotsample(clay_percent);
            CREATE INDEX IF NOT EXISTS idx_plotsample_silt_percent ON public.plotsample(silt_percent);
            CREATE INDEX IF NOT EXISTS idx_plotsample_sand_percent ON public.plotsample(sand_percent);
            CREATE INDEX IF NOT EXISTS idx_plotsample_fe_ug_per_g ON public.plotsample(fe_ug_per_g);
            CREATE INDEX IF NOT EXISTS idx_plotsample_na_ug_per_g ON public.plotsample(na_ug_per_g);
            CREATE INDEX IF NOT EXISTS idx_plotsample_al_ug_per_g ON public.plotsample(al_ug_per_g);
            CREATE INDEX IF NOT EXISTS idx_plotsample_k_ug_per_g ON public.plotsample(k_ug_per_g);
            CREATE INDEX IF NOT EXISTS idx_plotsample_ca_ug_per_g ON public.plotsample(ca_ug_per_g);
            CREATE INDEX IF NOT EXISTS idx_plotsample_mg_ug_per_g ON public.plotsample(mg_ug_per_g);
            CREATE INDEX IF NOT EXISTS idx_plotsample_mn_ug_per_g ON public.plotsample(mn_ug_per_g);
            CREATE INDEX IF NOT EXISTS idx_plotsample_s_ug_per_g ON public.plotsample(s_ug_per_g);
            CREATE INDEX IF NOT EXISTS idx_plotsample_cl_ug_per_g ON public.plotsample(cl_ug_per_g);
            CREATE INDEX IF NOT EXISTS idx_plotsample_p_ug_per_g ON public.plotsample(p_ug_per_g);
            CREATE INDEX IF NOT EXISTS idx_plotsample_si_ug_per_g ON public.plotsample(si_ug_per_g);
            CREATE INDEX IF NOT EXISTS idx_plotsample_subsample_replica_weight ON public.plotsample(subsample_replica_weight);
            CREATE INDEX IF NOT EXISTS idx_plotsample_created_on ON public.plotsample(created_on);
            CREATE INDEX IF NOT EXISTS idx_plotsample_last_updated ON public.plotsample(last_updated);
            CREATE INDEX IF NOT EXISTS idx_plotsample_fungi_per_g ON public.plotsample(fungi_per_g);
            CREATE INDEX IF NOT EXISTS idx_plotsample_bacteria_per_g ON public.plotsample(bacteria_per_g);
            CREATE INDEX IF NOT EXISTS idx_plotsample_archea_per_g ON public.plotsample(archea_per_g);
            CREATE INDEX IF NOT EXISTS idx_plotsample_methanogens_per_g ON public.plotsample(methanogens_per_g);
            CREATE INDEX IF NOT EXISTS idx_plotsample_methanotrophs_per_g ON public.plotsample(methanotrophs_per_g);
            CREATE INDEX IF NOT EXISTS idx_plotsample_replicate ON public.plotsample(replicate);

            -- TRANSECT table indexes
            CREATE INDEX IF NOT EXISTS idx_transect_name ON public.transect(name);
            CREATE INDEX IF NOT EXISTS idx_transect_description ON public.transect(description);
            CREATE INDEX IF NOT EXISTS idx_transect_date_created ON public.transect(date_created);
            CREATE INDEX IF NOT EXISTS idx_transect_last_updated ON public.transect(last_updated);

            -- TRANSECTNODE table indexes
            CREATE INDEX IF NOT EXISTS idx_transectnode_order ON public.transectnode("order");

            -- SENSOR table indexes
            CREATE INDEX IF NOT EXISTS idx_sensor_name ON public.sensor(name);
            CREATE INDEX IF NOT EXISTS idx_sensor_description ON public.sensor(description);
            CREATE INDEX IF NOT EXISTS idx_sensor_comment ON public.sensor(comment);
            CREATE INDEX IF NOT EXISTS idx_sensor_last_updated ON public.sensor(last_updated);
            CREATE INDEX IF NOT EXISTS idx_sensor_serial_number ON public.sensor(serial_number);
            CREATE INDEX IF NOT EXISTS idx_sensor_manufacturer ON public.sensor(manufacturer);

            -- PLOTSENSORASSIGNMENTS table indexes
            CREATE INDEX IF NOT EXISTS idx_plotsensorassignments_date_from ON public.plotsensorassignments(date_from);
            CREATE INDEX IF NOT EXISTS idx_plotsensorassignments_date_to ON public.plotsensorassignments(date_to);
            CREATE INDEX IF NOT EXISTS idx_plotsensorassignments_depth_cm ON public.plotsensorassignments(depth_cm);

            -- SENSORDATA table indexes
            CREATE INDEX IF NOT EXISTS idx_sensordata_instrument_seq ON public.sensordata(instrument_seq);
            CREATE INDEX IF NOT EXISTS idx_sensordata_time_zone ON public.sensordata(time_zone);
            CREATE INDEX IF NOT EXISTS idx_sensordata_temperature_1 ON public.sensordata(temperature_1);
            CREATE INDEX IF NOT EXISTS idx_sensordata_temperature_2 ON public.sensordata(temperature_2);
            CREATE INDEX IF NOT EXISTS idx_sensordata_temperature_3 ON public.sensordata(temperature_3);
            CREATE INDEX IF NOT EXISTS idx_sensordata_soil_moisture_count ON public.sensordata(soil_moisture_count);
            CREATE INDEX IF NOT EXISTS idx_sensordata_shake ON public.sensordata(shake);
            CREATE INDEX IF NOT EXISTS idx_sensordata_error_flat ON public.sensordata(error_flat);
            CREATE INDEX IF NOT EXISTS idx_sensordata_last_updated ON public.sensordata(last_updated);
            CREATE INDEX IF NOT EXISTS idx_sensordata_time_utc ON public.sensordata(time_utc);
            CREATE INDEX IF NOT EXISTS idx_sensordata_temperature_average ON public.sensordata(temperature_average);

            -- SOILPROFILE table indexes
            CREATE INDEX IF NOT EXISTS idx_soilprofile_name ON public.soilprofile(name);
            CREATE INDEX IF NOT EXISTS idx_soilprofile_gradient ON public.soilprofile(gradient);
            CREATE INDEX IF NOT EXISTS idx_soilprofile_weather ON public.soilprofile(weather);
            CREATE INDEX IF NOT EXISTS idx_soilprofile_topography ON public.soilprofile(topography);
            CREATE INDEX IF NOT EXISTS idx_soilprofile_vegetation_type ON public.soilprofile(vegetation_type);
            CREATE INDEX IF NOT EXISTS idx_soilprofile_aspect ON public.soilprofile(aspect);
            CREATE INDEX IF NOT EXISTS idx_soilprofile_lythology_surficial_deposit ON public.soilprofile(lythology_surficial_deposit);
            CREATE INDEX IF NOT EXISTS idx_soilprofile_created_on ON public.soilprofile(created_on);
            CREATE INDEX IF NOT EXISTS idx_soilprofile_last_updated ON public.soilprofile(last_updated);
            CREATE INDEX IF NOT EXISTS idx_soilprofile_soil_diagram ON public.soilprofile(soil_diagram);
            CREATE INDEX IF NOT EXISTS idx_soilprofile_photo ON public.soilprofile(photo);
            CREATE INDEX IF NOT EXISTS idx_soilprofile_parent_material ON public.soilprofile(parent_material);
            CREATE INDEX IF NOT EXISTS idx_soilprofile_coord_x ON public.soilprofile(coord_x);
            CREATE INDEX IF NOT EXISTS idx_soilprofile_coord_y ON public.soilprofile(coord_y);
            CREATE INDEX IF NOT EXISTS idx_soilprofile_coord_z ON public.soilprofile(coord_z);
            CREATE INDEX IF NOT EXISTS idx_soilprofile_coord_srid ON public.soilprofile(coord_srid);

            -- ===============================================
            -- End of migration script
            -- ===============================================

        "#;

        db.execute_unprepared(create_all_tables).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Drop tables in reverse order of creation, so references are removed cleanly
        let drop_all = r#"
            DROP TABLE IF EXISTS public.soilprofile;
            DROP TABLE IF EXISTS public.plotsensorassignments;
            DROP TABLE IF EXISTS public.sensor;
            DROP TABLE IF EXISTS public.transectnode;
            DROP TABLE IF EXISTS public.transect;
            DROP TABLE IF EXISTS public.plotsample;
            DROP TABLE IF EXISTS public.plot;
            DROP TABLE IF EXISTS public.instrumentexperimentchannel;
            DROP TABLE IF EXISTS public.instrumentexperiment;
            DROP TABLE IF EXISTS public.gnss;
            DROP TABLE IF EXISTS public.area;
            DROP TABLE IF EXISTS public.soiltype;
            DROP TABLE IF EXISTS public.project;
            DROP TYPE IF EXISTS public.gradientchoices;
            DROP EXTENSION IF EXISTS postgis;
        "#;

        db.execute_unprepared(drop_all).await?;
        Ok(())
    }
}
