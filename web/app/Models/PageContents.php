<?php
declare(strict_types=1);

namespace Wikijump\Models;

use Illuminate\Database\Eloquent\Model;
use Ozone\Framework\Database\Criteria;
use Wikidot\DB\PageRevision;
use Wikidot\DB\PageRevisionPeer;

class PageContents extends Model
{
    /**
     * The attributes that are mass assignable.
     *
     * @var array
     */
    protected $fillable = [
        'revision_id',
        'wikitext',
        'compiled_html',
        'generator',
    ];

    private static function getLatestRevision(string $page_id): PageRevision
    {
        $c = new Criteria();
        $c->add('page_id', $page_id);
        $c->addOrderDescending('revision_id');
        return PageRevisionPeer::instance()->selectOneByCriteria($c);
    }

    public static function getLatestWikitext(string $page_id): string
    {
        $revision_id = self::getLatestRevision($page_id)->getRevisionId();
        return PageContents::select('wikitext')->find($revision_id);
    }

    public static function getLatestCompiledHtml(string $page_id): string
    {
        $revision_id = self::getLatestRevision($page_id)->getRevisionId();
        return PageContents::select('compiled_html')->find($revision_id);
    }
}
