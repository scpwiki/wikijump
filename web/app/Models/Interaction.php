<?php
declare(strict_types=1);

namespace Wikijump\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\MorphTo;
use Illuminate\Database\QueryException;
use Illuminate\Support\Facades\Log;
use Wikijump\Helpers\InteractionType;

/**
 * Class Interaction
 * Core class for creating persistent relationships of various types between two
 *  objects, such as a user following a page, or a site banning a user.
 * @property array|null metadata
 * @package Wikijump\Models
 * @method static where(array $array)
 */
class Interaction extends Model
{
    use HasFactory;

    /**
     * Indicates if the model should be timestamped.
     *
     * @var bool
     */
    public $timestamps = false;

    protected $guarded = [];

    /**
     * Metadata is stored in the database as JSON objects.
     * When we retrieve it, we want to cast it as an associative array.
     * Otherwise we'll just end up calling json_decode anyway. This also keeps
     *  us from having to manually json_encode the metadata before saving.
     * @var array
     */
    protected $casts = [
        'metadata' => 'array'
    ];


    /**
     * Find the parent object for a given setting.
     * @return MorphTo
     */
    public function setter() : MorphTo
    {
        return $this->morphTo();
    }

    /**
     * Find the target objects from an instance.
     * @return MorphTo
     */
    public function target() : MorphTo
    {
        return $this->morphTo();
    }

    /**
     * Check if an Interaction method call has valid datatypes.
     * @param $setter
     * @param $relation
     * @param $target
     * @param array|null $metadata
     * @return bool
     */
    public static function isInvalid($setter, $relation, $target, ?array $metadata = []) : bool
    {
        return !(
            is_subclass_of($setter, 'Illuminate\Database\Eloquent\Model')
            && InteractionType::isValue($relation)
            && is_subclass_of($target, 'Illuminate\Database\Eloquent\Model')
            && is_array($metadata)
        );
    }

    /**
     * Create and persist a new Interaction object.
     * @param $setter
     * @param int $relation
     * @param $target
     * @param array|null $metadata
     * @return bool
     */
    public static function create($setter, int $relation, $target, ?array $metadata = []) : bool
    {
        if(self::isInvalid($setter, $relation, $target, $metadata))
        {
            Log::error("Interaction::add was given an invalid set of params. 
            Setter: $setter Relation: $relation Target: $target Metadata: $metadata");
            abort(500);
            return false;
        }

        $interaction = new Interaction(
            [
                'setter_type' => get_class($setter),
                'setter_id' => $setter->id,
                'interaction_type' => $relation,
                'target_type' => get_class($target),
                'target_id' => $target->id,
                'metadata' => $metadata
            ]
        );

        try {
            return $interaction->save();
        }
        catch(QueryException $e) {
            /** Postgres unique constraint violation: */
            if($e->errorInfo[0] == 23505) {
                /**
                 * We'll want to throw something here that a controller can
                 * catch and return data to the user. Pending API work.
                 */
                return false;
            }
        }
        return false;
    }

    /**
     * Retrieve the actual Interaction object rather than the downstream objects.
     * @param $setter
     * @param $relation
     * @param $target
     * @return Interaction|null
     */
    public static function retrieve($setter, $relation, $target) : ?Interaction
    {
        if(self::isInvalid($setter, $relation, $target))
        {
            Log::error("Interaction::retrieve was given an invalid set of params. 
            Setter: $setter Relation: $relation Target: $target");
            abort(500);
            return null;
        }
        return Interaction::where(
            [
                'setter_type' => get_class($setter),
                'setter_id' => $setter->id,
                'interaction_type' => $relation,
                'target_type' => get_class($target),
                'target_id' => $target->id
            ]
        )->first();
    }

    /**
     * Update the metadata on an Interaction.
     * @param $setter
     * @param int $relation
     * @param $target
     * @param array $metadata
     * @return bool
     */
    public static function set($setter, int $relation, $target, array $metadata = []) : bool
    {
        if(self::isInvalid($setter, $relation, $target, $metadata))
        {
            Log::error("Interaction::set was given an invalid set of params. 
            Setter: $setter Relation: $relation Target: $target Metadata: $metadata");
            abort(500);
            return false;
        }

        if(self::check($setter, $relation, $target) === false) { return false; }

        /** @var Interaction $interaction */
        $interaction = Interaction::where(
            [
                'setter_type' => get_class($setter),
                'setter_id' => $setter->id,
                'interaction_type' => $relation,
                'target_type' => get_class($target),
                'target_id' => $target->id
            ]
        )->first();

        $interaction->metadata  = $metadata;

        return $interaction->save();
    }

    /**
     * Delete an interaction from the table.
     * @param $setter
     * @param int $relation
     * @param $target
     * @return bool
     */
    public static function remove($setter, int $relation, $target) : bool
    {
        if(self::isInvalid($setter, $relation, $target))
        {
            Log::error("Interaction::remove was given an invalid set of params. 
            Setter: $setter Relation: $relation Target: $target");
            abort(500);
            return false;
        }
            $interaction = Interaction::where(
                [
                    'setter_type' => get_class($setter),
                    'setter_id' => $setter->id,
                    'interaction_type' => $relation,
                    'target_type' => get_class($target),
                    'target_id' => $target->id
                ]
            );
            return (bool)$interaction->delete();
    }

    /**
     * Determine if a particular relationship exists between a source and target.
     * @param $setter
     * @param int $relation
     * @param $target
     * @return bool
     */
    public static function check($setter, int $relation, $target) : bool
    {
        if(self::isInvalid($setter, $relation, $target))
        {
            Log::error("Interaction::remove was given an invalid set of params. 
            Setter: $setter Relation: $relation Target: $target");
            abort(500);
            return false;
        }
        return (bool)Interaction::where(
            [
                'setter_type' => get_class($setter),
                'setter_id' => $setter->id,
                'interaction_type' => $relation,
                'target_type' => get_class($target),
                'target_id' => $target->id
            ]
        )->count();
    }
}
