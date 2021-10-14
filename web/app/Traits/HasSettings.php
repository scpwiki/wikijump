<?php
declare(strict_types=1);

namespace Wikijump\Traits;

use Illuminate\Database\Eloquent\Relations\MorphOne;
use Wikijump\Models\Settings;

trait HasSettings
{
    abstract public function defaults(): array;

    /**
     * Add the relationship from the parent class to a Settings object.
     * @return MorphOne
     * @see Settings
     */
    public function settings(): MorphOne
    {
        return $this->morphOne(Settings::class, 'setter');
    }

    /**
     * Update settings. New up a model if nonexistent.
     * @param array $settings A list of settings to update.
     * @return bool False would indicate a failure saving.
     */
    public function set(array $settings): bool
    {
        return $this->settings()
            ->firstOrNew()
            ->modify($settings);
    }

    /**
     * Get a single setting back, or the default if unset.
     * Note: If you want *all* the settings, use something like
     * $user->settings()->firstOrNew() instead.
     * @param string $setting
     * @return mixed
     */
    public function get(string $setting)
    {
        return $this->settings()
            ->firstOrNew()
            ->retrieve($setting);
    }
}
