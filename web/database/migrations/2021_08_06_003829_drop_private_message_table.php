<?php
declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

/**
 * Drop the Legacy private_message table.
 */
class DropPrivateMessageTable extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        Schema::dropIfExists('private_message');
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        Schema::create('private_message', function (Blueprint $table) {
            $table->id('message_id');
            $table->unsignedInteger('from_user_id')->nullable()->index();
            $table->unsignedInteger('to_user_id')->nullable()->index();
            $table->string('subject', 256)->nullable();
            $table->string('body', 200000)->nullable();
            $table->timestamp('date')->nullable();
            $table->unsignedInteger('flag')->nullable();
            $table->boolean('flag_new')->default(true);
        });
    }
}
