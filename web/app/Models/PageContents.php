<?php
declare(strict_types=1);

namespace Wikijump\Models;

use Illuminate\Database\Eloquent\Model;
use Wikijump\Traits\LegacyCompatibility;

class PageContents extends Model
{
    use LegacyCompatibility;

    /**
     * Override the primary key column name.
     *
     * @var string
     */
    protected $primaryKey = 'revision_id';

    /**
     * The attributes that are mass assignable.
     *
     * @var array
     */
    protected $fillable = ['revision_id', 'wikitext', 'compiled_html', 'generator'];
}
