<?php
declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class TagConfiguration extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        Schema::table('site', function (Blueprint $table) {
            $table->dropColumn('enable_tag_engine');
        });

        Schema::table('tag_settings', function (Blueprint $table) {
            $table->dropColumn('allowed_tags');
            $table->json('configuration_data')->default('{}');
        });
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        Schema::table('site', function (Blueprint $table) {
            $table->boolean('enable_tag_engine')->default(false);
        });

        Schema::table('tag_settings', function (Blueprint $table) {
            $table->dropColumn('configuration_data');
            $table->jsonb('allowed_tags')->default('[]');
        });
    }
}
