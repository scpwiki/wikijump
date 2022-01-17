<?php
declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class PageContents extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        // Create new contents table
        Schema::create('page_contents', function (Blueprint $table) {
            $table->timestamps();
            $table->foreignId('revision_id');
            $table->longtext('wikitext');
            $table->longtext('compiled_html');
            $table->string('generator', 64);

            $table->foreign('revision_id')->references('revision_id')->on('page_revision');
            $table->primary('revision_id');
        });

        // Migrate existing sources
        $revisions = DB::table('page_revision')
            ->select('revision_id', 'page_id', 'source_id')
            ->get()
            ->toArray();

        foreach ($revisions as $revision) {
            $wikitext = DB::table('page_source')
                ->where('source_id', $revision->source_id)
                ->pluck('text')
                ->first();

            $compiled_html = DB::table('page_compiled')
                ->where('page_id', $revision->page_id)
                ->pluck('text')
                ->first();

            DB::table('page_contents')->insert([
                'revision_id' => $revision->revision_id,
                'wikitext' => $wikitext ?? '',
                'compiled_html' => $compiled_html ?? '',
                'generator' => 'Text_Wiki (legacy)',
            ]);
        }

        // Drop old contents tables / columns
        Schema::table('page_revision', function (Blueprint $table) {
            $table->dropColumn('source_id');
        });

        Schema::table('page', function (Blueprint $table) {
            $table->dropColumn('source_id');
        });

        Schema::drop('page_source');
        Schema::drop('page_compiled');
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        Schema::drop('page_contents');

        Schema::create('page_source', function (Blueprint $table) {
            $table->id('source_id')->startingValue(63);
            $table->string('text', 200000)->nullable();
        });

        Schema::create('page_compiled', function (Blueprint $table) {
            $table->unsignedInteger('page_id')->primary();
            $table->string('text', 200000)->nullable();
            $table->timestamp('date_compiled')->nullable();
        });

        Schema::table('page_revision', function (Blueprint $table) {
            $table->unsignedInteger('source_id')->nullable();
        });

        Schema::table('page', function (Blueprint $table) {
            $table->unsignedInteger('source_id')->nullable();
        });
    }
}
