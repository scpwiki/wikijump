<?php
declare(strict_types=1);

namespace Wikijump\Models;

use Illuminate\Database\Eloquent\Model;
use Wikijump\Services\TagEnforcement\TagConfiguration;

class TagSettings extends Model
{
    /**
     * The attributes that are mass assignable.
     *
     * @var array
     */
    protected $fillable = ['site_id', 'configuration_name', 'configuration_data'];

    /**
     * Build a TagConfiguration corresponding to the data in this model.
     */
    public function getConfiguration(): TagConfiguration
    {
        return new TagConfiguration($this->configuration_data);
    }
}
