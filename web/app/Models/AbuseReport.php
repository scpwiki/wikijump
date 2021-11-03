<?php
declare(strict_types=1);

namespace Wikijump\Models;


class AbuseReport extends Model
{
    /**
     * The attributes that are mass assignable.
     *
     * @var array
     */
    protected $fillable = ['user_id', 'entity_id', 'reason'];

}
