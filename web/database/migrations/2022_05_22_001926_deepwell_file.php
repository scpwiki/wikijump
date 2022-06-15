<?php
declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class DeepwellFile extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        // This hands ownership of the file table to DEEPWELL

        Schema::drop('file');
        Schema::drop('files_event');

        // Create enum types for use in page_revision
        DB::statement("
            CREATE TYPE file_revision_type AS ENUM (
                'create',
                'update',
                'delete',
                'undelete'
            )
        ");

        DB::statement("
            CREATE TYPE file_revision_change AS ENUM (
                'name',
                'blob',
                'mime',
                'licensing'
            )
        ");

        // Create new tables
        DB::statement("
            CREATE TABLE file (
                file_id TEXT PRIMARY KEY,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
                updated_at TIMESTAMP WITH TIME ZONE,
                deleted_at TIMESTAMP WITH TIME ZONE,
                name TEXT NOT NULL,
                page_id BIGINT NOT NULL REFERENCES page(page_id),

                UNIQUE (page_id, name, deleted_at)
            )
        ");

        // Like page_revision, we have to use JSON instead of TEXT[]

        DB::statement("
            CREATE TABLE file_revision (
                revision_id BIGSERIAL PRIMARY KEY,
                revision_type file_revision_type NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
                revision_number INTEGER NOT NULL,
                file_id TEXT NOT NULL REFERENCES file(file_id),
                page_id BIGINT NOT NULL REFERENCES page(page_id),
                user_id BIGINT NOT NULL REFERENCES users(id),
                name TEXT NOT NULL,
                s3_hash BYTEA NOT NULL,
                mime_hint TEXT NOT NULL,
                size_hint BIGINT NOT NULL,
                licensing JSONB NOT NULL,
                changes JSON NOT NULL DEFAULT '[]', -- List of changes in this revision
                comments TEXT NOT NULL,
                hidden JSON NOT NULL DEFAULT '[]', -- List of fields to be hidden/suppressed

                CHECK (length(name) > 0 AND length(name) < 256),  -- Constrain filename length
                CHECK (length(s3_hash) = 64),                     -- SHA-512 hash size
                CHECK (mime_hint != ''),                          -- Should have a MIME hint

                -- NOTE: json_array_to_text_array() is needed while we're still on JSON

                -- Ensure array only contains valid values
                -- Change this to use the 'page_revision_change' type later
                CHECK (json_array_to_text_array(changes) <@ '{
                    \"page\",
                    \"name\",
                    \"blob\",
                    \"mime\",
                    \"licensing\"
                }'),

                -- Ensure first revision reports all changes
                --
                -- This is implemented  by seeing if it's a superset or equal to all valid values.
                -- Since we already check if it's a subset or equal, this is the same as
                -- strict equivalence, but without regard for ordering.
                CHECK (
                    revision_type != 'create' OR
                    json_array_to_text_array(changes) @> '{
                        \"page\",
                        \"name\",
                        \"blob\",
                        \"mime\",
                        \"licensing\"
                    }'
                ),

                -- Ensure array is not empty for update revisions
                CHECK (revision_type != 'update' OR json_array_to_text_array(changes) != '{}'),

                -- Ensure page creations are always the first revision
                CHECK (revision_number != 0 OR revision_type = 'create'),

                -- For logical consistency, and adding an index
                UNIQUE (file_id, page_id, revision_number)
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
        Schema::drop('file');
        Schema::drop('file_revision');

        Schema::create('file', function (Blueprint $table) {
           $table->id('file_id');
           $table->unsignedInteger('page_id')->nullable()->index();
           $table->unsignedInteger('site_id')->nullable()->index();
           $table->string('filename', 100)->nullable();
           $table->string('mimetype', 100)->nullable();
           $table->string('description' ,200)->nullable();
           $table->string('description_short', 200)->nullable();
           $table->string('comment', 400)->nullable();
           $table->unsignedInteger('size')->nullable();
           $table->timestamp('date_added')->nullable();
           $table->unsignedInteger('user_id')->nullable();
           $table->string('user_string', 80)->nullable();
           $table->boolean('has_resized')->default(false);
        });

        Schema::create('files_event', function (Blueprint $table) {
           $table->id('file_event_id');
           $table->string('filename', 100)->nullable();
           $table->timestamp('date')->nullable();
           $table->unsignedInteger('user_id')->nullable();
           $table->string('user_string', 80)->nullable();
           $table->string('action', 80)->nullable();
           $table->string('action_extra', 80)->nullable();
        });
    }
}
