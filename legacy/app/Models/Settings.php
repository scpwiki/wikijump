<?php
declare(strict_types=1);

namespace Wikijump\Models;

use Exception;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\MorphTo;

/**
 * Class Settings
 * @package Wikijump\Models
 * @mixin Builder
 */
class Settings extends Model
{
    use HasFactory;

    public array $defaults;

    /**
     * Indicates if the model should be timestamped.
     *
     * @var bool
     */
    public $timestamps = false;

    /**
     * Settings are stored in the database as JSON objects.
     * When we retrieve them, we want to cast them as an associative array.
     * Otherwise we'll just end up calling json_decode anyway. This also keeps
     *  us from having to manually json_encode the settings before saving.
     * @var array
     */
    protected $casts = [
        'settings' => 'array',
    ];

    /**
     * Find the parent object for a given setting.
     * @return MorphTo
     */
    public function setter(): MorphTo
    {
        return $this->morphTo();
    }

    /**
     * The Settings::modify() method will instead delete the corresponding key
     *  from the object if the new value matches the default. If there are no
     *  remaining non-default settings stored, it will destroy the object.
     * Note that it will only update any keys that are passed to it, and leave
     *  the others alone.
     * Example usage:
     * $user->settings->modify(['allow_pms' => false, 'invisible', => true]);
     * There is also a shortcut on the Setter models, e.g.:
     * $user->set(['allow_pms' => false, 'invisible', => true]);
     * TODO: Add handling for e.g., a user's per-site settings.
     * @param array $setting
     * @param array $options
     * @return bool
     * @throws Exception
     * @see Model::update()
     */
    public function modify(array $setting = [], array $options = []): bool
    {
        $this->defaults = $this->setter->defaults();

        /**
         * You can't modify the property array directly, only assign a new value.
         */
        $currentSettings = $this->settings ?? [];

        foreach ($setting as $key => $value) {
            if (isset($this->defaults["$key"]) === false) {
                throw new Exception("$key does not have a default set");
            }
            if ($this->defaults["$key"] === $value) {
                unset($currentSettings[$key]);
            } else {
                $currentSettings["$key"] = $value;
            }
        }
        $this->settings = $currentSettings;
        if ($this->settings == []) {
            if ($this->id == null) {
                /**
                 * If the model doesn't exist and no changes were made from default,
                 * there's nothing to save, but any caller doesn't need to know that.
                 */
                return true;
            }
            /**
             * If the model *did* exist and all the settings are no defaults,
             * destroy the object. Returns true.
             */
            Settings::destroy($this->id);
            return true;
        }
        /**
         * Otherwise, save the changes.
         */
        return $this->save($options);
    }

    /**
     * Get a single setting back from the settings. If there's no matching
     *  setting set, use the default.
     * Note: If you want *all* the settings, use $user->settings()->firstOrNew()
     *  instead.
     * @param string $setting
     * @return mixed
     */
    public function retrieve(string $setting)
    {
        $this->defaults = $this->setter->defaults();
        if (isset($this->settings[$setting])) {
            return $this->settings[$setting];
        } else {
            return $this->defaults[$setting];
        }
    }
}
