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

        DB::statement("
            CREATE TABLE file (
                file_id TEXT PRIMARY KEY,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
                updated_at TIMESTAMP WITH TIME ZONE,
                deleted_at TIMESTAMP WITH TIME ZONE,
                name TEXT NOT NULL,
                s3_hash BYTEA,  -- Nullable to allow for hard deletions
                user_id BIGINT NOT NULL REFERENCES users(id),
                page_id BIGINT NOT NULL REFERENCES page(page_id),
                size_hint BIGINT NOT NULL,
                mime_hint TEXT NOT NULL,
                licensing JSONB NOT NULL,

                UNIQUE (page_id, name, deleted_at),               -- Names are scoped per-page
                CHECK (length(name) > 0 AND length(name) < 256),  -- Constrain filename length
                CHECK (length(s3_hash) = 64),                     -- SHA-512 hash size
                CHECK ((s3_hash IS NULL) = (size_hint = 0))       -- Hard deletion consistency
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
