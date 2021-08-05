<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class CreateInteractionsTable extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        Schema::create('interactions', function (Blueprint $table) {
            $table->id();
            $table->string('setter_type', 64);
            $table->unsignedInteger('setter_id');
            $table->unsignedSmallInteger('interaction_type');
            $table->string('target_type', 64);
            $table->unsignedInteger('target_id');
            $table->json('metadata')->nullable();
            $table->timestamp('created_at')->useCurrent();

            $table->index('setter_id');
            $table->index('target_id');
            $table->unique(['setter_type', 'setter_id', 'interaction_type', 'target_type', 'target_id']);
        });
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        Schema::dropIfExists('interactions');
    }
}
