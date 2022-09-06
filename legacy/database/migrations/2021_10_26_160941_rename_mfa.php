<?php
declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class RenameMfa extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        Schema::table('users', function (Blueprint $table) {
            $table->renameColumn('two_factor_secret', 'multi_factor_secret');
            $table->renameColumn('two_factor_recovery_codes', 'multi_factor_recovery_codes');
        });
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        Schema::table('users', function (Blueprint $table) {
            $table->renameColumn('multi_factor_secret', 'two_factor_secret');
            $table->renameColumn('multi_factor_recovery_codes', 'two_factor_recovery_codes');
        });
    }
}
