<?php
declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class FilePageRevision extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        // Add revision types
        DB::statement("ALTER TYPE revision_type ADD VALUE IF NOT EXISTS 'file_create'");
        DB::statement("ALTER TYPE revision_type ADD VALUE IF NOT EXISTS 'file_update'");
        DB::statement("ALTER TYPE revision_type ADD VALUE IF NOT EXISTS 'file_delete'");

        // Add file_id to revision table
        DB::statement("
            -- Nullable since most revisions don't affect files
            ALTER TABLE page_revision ADD COLUMN file_id TEXT REFERENCES file(file_id)
        ");

        // Update check constraint
        DB::statement("ALTER TYPE revision_change ADD VALUE IF NOT EXISTS 'file'");
        DB::statement("ALTER TABLE page_revision DROP CONSTRAINT page_revision_changes_check");
        DB::statement("
            -- Ensure array only contains valid values
            ALTER TABLE page_revision
                ADD CONSTRAINT page_revision_changes_check
                CHECK (json_array_to_text_array(changes) <@ '{
                    \"wikitext\",
                    \"title\",
                    \"alt_title\",
                    \"slug\",
                    \"tags\",
                    \"file\"
                }')
        ");
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        // Drop file_id from revision table
        DB::statement("ALTER TABLE page_revision DROP COLUMN file_id");

        // Undo check constraint
        DB::statement("ALTER TABLE page_revision DROP CONSTRAINT page_revision_changes_check");
        DB::statement("
            -- Ensure array only contains valid values
            ALTER TABLE page_revision
                ADD CONSTRAINT page_revision_changes_check
                CHECK (json_array_to_text_array(changes) <@ '{
                    \"wikitext\",
                    \"title\",
                    \"alt_title\",
                    \"slug\",
                    \"tags\"
                }')
        ");
    }
}
