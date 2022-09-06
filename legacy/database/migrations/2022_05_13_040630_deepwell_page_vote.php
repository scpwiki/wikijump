<?php
declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class DeepwellPageVote extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        // This hands ownership of the page vote table to DEEPWELL

        // Drop old tables
        Schema::drop('page_rate_vote');

        // Create new table
        DB::statement("
            CREATE TABLE page_vote (
                page_vote_id BIGSERIAL PRIMARY KEY,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
                deleted_at TIMESTAMP WITH TIME ZONE,
                disabled_at TIMESTAMP WITH TIME ZONE,
                disabled_by BIGINT REFERENCES users(id),
                page_id BIGINT NOT NULL REFERENCES page(page_id),
                user_id BIGINT NOT NULL REFERENCES users(id),
                value SMALLINT NOT NULL,

                UNIQUE (page_id, user_id, deleted_at),
                CHECK ((disabled_at IS NULL) = (disabled_by IS NULL))
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
        Schema::drop('page_vote');
        Schema::create('page_rate_vote', function (Blueprint $table) {
            $table->id('rate_id');
            $table->unsignedInteger('user_id')->nullable();
            $table->unsignedInteger('page_id')->nullable();
            $table->integer('rate')->default(1);
            $table->timestamp('date')->nullable();

            $table->unique(['user_id', 'page_id']);
        });
    }
}
