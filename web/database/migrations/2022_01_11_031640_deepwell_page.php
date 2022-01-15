<?php
declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

function find(array $array, string $field, $value)
{
    foreach ($array as $item) {
        if ($item->$field === $value) {
            return $item;
        }
    }

    throw new Error("Cannot find item in array where field $field has value $value");
}

class DeepwellPage extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        // This hands ownership of the primary page tables to DEEPWELL

        Schema::rename('page', 'page_old');
        Schema::rename('page_revision', 'page_revision_old');
        Schema::rename('page_metadata', 'page_metadata_old');

        DB::statement("
            CREATE TABLE page (
                page_id BIGSERIAL PRIMARY KEY,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
                updated_at TIMESTAMP WITH TIME ZONE,
                deleted_at TIMESTAMP WITH TIME ZONE,
                site_id BIGINT NOT NULL REFERENCES site(site_id),
                page_category_id BIGINT NOT NULL REFERENCES category(category_id),
                slug TEXT NOT NULL,
                discussion_thread_id BIGINT REFERENCES forum_thread(thread_id),

                UNIQUE (site_id, slug)
            )
        ");

        DB::statement("
            CREATE TABLE page_revision (
                revision_id BIGSERIAL PRIMARY KEY,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
                revision_number INT NOT NULL,
                page_id BIGINT NOT NULL REFERENCES page(page_id),
                site_id BIGINT NOT NULL REFERENCES site(site_id),
                user_id BIGINT NOT NULL REFERENCES users(id),
                wikitext_hash BYTEA NOT NULL REFERENCES text(hash),
                compiled_hash BYTEA REFERENCES text(hash),
                compiled_at TIMESTAMP WITH TIME ZONE,
                compiled_generator TEXT,
                comments TEXT NOT NULL,
                comments_edited_at TIMESTAMP WITH TIME ZONE,
                comments_edited_by BIGINT REFERENCES users(id),
                hidden TEXT[] NOT NULL DEFAULT '{}', -- List of fields to be hidden/suppressed
                title TEXT NOT NULL,
                alt_title TEXT,
                slug TEXT NOT NULL,
                tags TEXT[] NOT NULL DEFAULT '{}', -- Should be sorted and deduplicated before insertion
                metadata JSONB NOT NULL DEFAULT '{}', -- Customizable metadata. Currently unused.

                -- Ensure all compiled fields are null, or all are non-null
                CHECK ((compiled_hash IS NULL) = (compiled_at IS NULL)),
                CHECK ((compiled_at IS NULL) = (compiled_generator IS NULL)),

                -- Ensure both comments_edited fields are null, or both are non-null
                CHECK ((comments_edited_at IS NULL) = (comments_edited_by IS NULL)),

                UNIQUE (page_id, site_id, revision_number)
            )
        ");

        DB::statement("
            CREATE TABLE page_parent (
                parent_page_id BIGINT REFERENCES page(page_id),
                child_page_id BIGINT REFERENCES page(page_id),
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),

                PRIMARY KEY (parent_page_id, child_page_id)
            )
        ");

        DB::statement("
            CREATE TABLE page_attribution (
                page_id BIGINT REFERENCES page(page_id),
                user_id BIGINT REFERENCES users(id),
                -- Text enum describing the kind of attribution
                -- Currently synced to Crom: 'author', 'rewrite', 'translator', 'maintainer'
                attribution_type TEXT NOT NULL,
                attribution_date DATE NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),

                PRIMARY KEY (page_id, user_id, attribution_type, attribution_date)
            )
        ");

        DB::statement("
            CREATE TABLE page_lock (
                page_lock_id BIGSERIAL PRIMARY KEY,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
                updated_at TIMESTAMP WITH TIME ZONE,
                deleted_at TIMESTAMP WITH TIME ZONE,
                -- Text enum describing what kind of lock (e.g. authors only, staff only)
                -- Currently the only value is 'wikidot' (meaning mods+ only)
                lock_type TEXT NOT NULL,
                page_id BIGINT NOT NULL REFERENCES page(page_id),
                user_id BIGINT NOT NULL REFERENCES users(id),
                reason TEXT NOT NULL,

                UNIQUE (page_id, deleted_at)
            )
        ");

        // Migrate page data
        $pages = DB::table('page_old')
            ->select(
                'page_id',
                'site_id',
                'category_id',
                'parent_page_id',
                'revision_id',
                'metadata_id',
                'revision_number',
                'title',
                'unix_name',
                'date_created',
                'date_last_edited',
                'thread_id',
                'blocked',
                'tags',
            )
            ->get()
            ->toArray();

        $max_page_id = 0;
        foreach ($pages as $page) {
            DB::insert('
                INSERT INTO page (
                    page_id,
                    created_at,
                    updated_at,
                    site_id,
                    page_category_id,
                    slug,
                    discussion_thread_id
                ) VALUES (?, ?, ?, ?, ?, ?, ?)
            ', [
                $page->page_id,
                $page->date_created,
                $page->date_last_edited,
                $page->site_id,
                $page->category_id,
                $page->unix_name,
                $page->thread_id,
            ]);

            $max_page_id = max($max_page_id, $page->page_id);
        }
        DB::statement('ALTER SEQUENCE page_page_id START WITH ?', [$max_page_id]);

        $page_revisions = DB::table('page_revision_old')
            ->select(
                'page_id',
                'metadata_id',
                'revision_number',
                'date_last_edited',
                'user_id',
                'comments',
                'site_id',
                'wikitext_hash',
                'compiled_hash',
                'compiled_generator',
            )
            ->get()
            ->toArray();

        $metadata_list = DB::table('page_metadata_old')
            ->select(
                'metadata_id',
                'parent_page_id',
                'title',
                'unix_name',
                'owner_user_id',
            )
            ->get()
            ->toArray();

        $max_revision_id = 0;
        foreach ($page_revisions as $revision) {
            $metadata = find($metadata_list, 'metadata_id', $revision->metadata_id);
            $page = find($pages, 'metadata_id', $revision->metadata_id);

            DB::insert('
                INSERT INTO page_revision (
                    revision_id,
                    created_at,
                    revision_number,
                    page_id,
                    site_id,
                    user_id,
                    wikitext_hash,
                    compiled_hash,
                    compiled_at,
                    compiled_generator,
                    comments,
                    title,
                    slug,
                    tags
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ', [
                $revision->revision_id,
                $revision->date_last_edited,
                $revision->revision_number,
                $revision->page_id,
                $revision->site_id,
                $revision->user_id,
                $revision->wikitext_hash,
                $revision->compiled_hash,
                $revision->compiled_generator,
                $revision->comments,
                $metadata->title,
                $page->tags,
            ]);

            $max_revision_id = max($max_revision_id, $revision->revision_id);
        }
        DB::statement('ALTER SEQUENCE page_revision_revision_id START WITH ?', [$max_revision_id]);

        // Fix foreign keys
        DB::statement('ALTER TABLE file DROP CONSTRAINT file_page_id_foreign');
        DB::statement('ALTER TABLE forum_thread DROP CONSTRAINT forum_thread_page_id_foreign');
        DB::statement('ALTER TABLE front_forum_feed DROP CONSTRAINT front_forum_feed_page_id_foreign');
        DB::statement('ALTER TABLE page_connection DROP CONSTRAINT page_connection_from_page_id_fkey');
        DB::statement('ALTER TABLE page_connection DROP CONSTRAINT page_connection_to_page_id_fkey');
        DB::statement('ALTER TABLE page_connection_missing DROP CONSTRAINT page_connection_missing_from_page_id_fkey');
        DB::statement('ALTER TABLE page_link DROP CONSTRAINT page_link_page_id_fkey');
        DB::statement('ALTER TABLE page_edit_lock DROP CONSTRAINT page_edit_lock_page_id_foreign');
        DB::statement('ALTER TABLE page_rate_vote DROP CONSTRAINT page_rate_vote_page_id_foreign');
        DB::statement('ALTER TABLE watched_page DROP CONSTRAINT watched_page_page_id_foreign');

        DB::statement('ALTER TABLE file ADD FOREIGN KEY page_id REFERENCES page(page_id)');
        DB::statement('ALTER TABLE forum_thread ADD FOREIGN KEY page_id REFERENCES page(page_id)');
        DB::statement('ALTER TABLE front_forum_feed ADD FOREIGN KEY page_id REFERENCES page(page_id)');
        DB::statement('ALTER TABLE page_connection ADD FOREIGN KEY from_page_id REFERENCES page(page_id)');
        DB::statement('ALTER TABLE page_connection ADD FOREIGN KEY to_page_id REFERENCES page(page_id)');
        DB::statement('ALTER TABLE page_connection_missing ADD FOREIGN KEY from_page_id REFERENCES page(page_id)');
        DB::statement('ALTER TABLE page_link ADD FOREIGN KEY page_id REFERENCES page(page_id)');
        DB::statement('ALTER TABLE page_edit_lock ADD FOREIGN KEY page_id REFERENCES page(page_id)');
        DB::statement('ALTER TABLE page_rate_vote ADD FOREIGN KEY page_id REFERENCES page(page_id)');
        DB::statement('ALTER TABLE watched_page ADD FOREIGN KEY page_id REFERENCES page(page_id)');

        // Drop old tables
        Schema::drop('page_old');
        Schema::drop('page_revision_old');
        Schema::drop('page_metadata_old');
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        // This is going to be a lot of boilerplate to do, so I'm going to skip it
    }
}
