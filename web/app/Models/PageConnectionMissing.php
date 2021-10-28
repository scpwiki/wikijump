<?php
declare(strict_types=1);

namespace Wikijump\Models;

use Illuminate\Contracts\Validation\Validator;
use Illuminate\Database\Eloquent\Model;

class PageConnectionMissing extends Model
{
    /**
     * Manually setting the name of the table.
     *
     * @var string
     */
    protected $table = 'page_connection_missing';

    /**
     * The attributes that are mass assignable.
     *
     * @var array
     */
    protected $fillable = [
        'from_page_id',
        'from_site_id',
        'to_page_name',
        'to_site_name',
        'connection_type',
        'count',
    ];

    /**
     * Creates a validator for this database object.
     *
     * @param array $data
     * @return Validator
     */
    protected function validator(array $data)
    {
        return Validator::make($data, [
            'count' => 'min:1',
        ]);
    }
}
