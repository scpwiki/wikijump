<?php
declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class SiteSlugNotNull extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        // Rename unix_name -> slug, make it not null
        Schema::table('site', function (Blueprint $table) {
            $table->text('unix_name')->nullable(false)->change();
            $table->renameColumn('unix_name', 'slug');
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
            $table->string('unix_name', 80)->nullable(true)->change();
            $table->renameColumn('slug', 'unix_name');
        });
    }
}
