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
     * Indicates if the model has default timestamp fields.
     *
     * @var boolean
     */
    public $timestamps = false;

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
     * The attributes that should be cast to native types.
     *
     * @var array
     */
    protected $casts = [
        'created_at' => 'datetime',
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
