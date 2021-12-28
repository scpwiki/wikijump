<?php
declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class DeepwellPageLink extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        // This hands ownership of these tables to DEEPWELL
        Schema::drop('page_link');
        Schema::drop('page_connection');
        Schema::drop('page_connection_missing');

        DB::statement("
            CREATE TABLE page_link (
                page_id BIGINT,
                url TEXT,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL,
                updated_at TIMESTAMP WITH TIME ZONE,
                count INT NOT NULL CHECK (count > 0),

                PRIMARY KEY (page_id, url)
            )
        ");

        DB::statement("
            CREATE TABLE page_connection (
                from_page_id BIGINT,
                to_page_id BIGINT,
                connection_type TEXT
                    CHECK (connection_type = ANY(ARRAY[
                        'include-messy',
                        'include-elements',
                        'component',
                        'link',
                        'redirect'
                    ])),
                created_at TIMESTAMP WITH TIME ZONE NOT NULL,
                updated_at TIMESTAMP WITH TIME ZONE,
                count INT NOT NULL CHECK (count > 0),

                PRIMARY KEY (from_page_id, to_page_id, connection_type)
            )
        ");

        DB::statement("
            CREATE TABLE page_connection_missing (
                from_page_id BIGINT,
                to_page_slug TEXT,
                connection_type TEXT
                    CHECK (connection_type = ANY(ARRAY[
                        'include-messy',
                        'include-elements',
                        'component',
                        'link',
                        'redirect'
                    ])),
                created_at TIMESTAMP WITH TIME ZONE NOT NULL,
                updated_at TIMESTAMP WITH TIME ZONE,
                count INT NOT NULL CHECK (count > 0),

                PRIMARY KEY (from_page_id, to_page_slug, connection_type)
            )
        ");
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        Schema::drop('page_link');
        Schema::drop('page_connection');
        Schema::drop('page_connection_missing');

        Schema::create('page_link', function (Blueprint $table) {
            $table->id();
            $table->timestamps();
            $table->foreignId('page_id')->index();
            $table->foreignId('site_id');
            $table->string('url');
            $table->unsignedSmallInteger('count');

            $table->unique(['page_id', 'site_id', 'url']);
        });

        Schema::create('page_connection', function (Blueprint $table) {
            $table->id();
            $table->timestamps();
            $table->foreignId('from_page_id')->index();
            $table->foreignId('from_site_id');
            $table->foreignId('to_page_id')->index();
            $table->foreignId('to_site_id');
            $table->unsignedSmallInteger('count');
            $table->enum('connection_type', ['include-messy', 'include-elements', 'component', 'link']);

            $table->unique(['from_page_id', 'from_site_id', 'to_page_id', 'to_site_id', 'connection_type']);
        });

        Schema::create('page_connection_missing', function (Blueprint $table) {
            $table->id();
            $table->timestamps();
            $table->foreignId('from_page_id')->index();
            $table->foreignId('from_site_id');
            $table->string('to_page_name')->index();
            $table->string('to_site_name')->nullable();
            $table->unsignedSmallInteger('count');
            $table->enum('connection_type', ['include-messy', 'include-elements', 'component', 'link']);

            $table->unique(['from_page_id', 'from_site_id', 'to_page_name', 'to_site_name']);
        });
    }
}
