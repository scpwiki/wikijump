<?php
declare(strict_types=1);

namespace Wikijump\Helpers;

use Wikijump\Models\User;

/** A collection of static methods to smooth the transition to Wikijump code. */
class LegacyTools
{

    /**
     * A function to take an absolute path to a file and transform it to a properly namespaced class.
     *
     * This may need to be adjusted occasionally if namespaces change (check composer.json) or things move around.
     *
     * @param string $path An absolute path e.g., /var/www/path/to/modules/file.php
     * @return string A namespaced legacy class e.g., Wikidot\Modules\File
     */
    public static function getNamespacedClassFromPath(string $path) : string
    {
        $offset = strlen(dirname(__FILE__, 3)); // Get the length of the string of the absolute path 3 levels up from this file.
        $unique_path = substr($path, $offset, -4); // Chop off that length and the last 4 characters. (.php)
        $unique_path = str_replace('/', '\\', $unique_path);
        $translations = [
            "\\php\\" => "Wikidot\\",
            "\\lib\\ozoneframework\\php\\core\\" => "Ozone\\Framework\\",
            "\\lib\\ozoneframework\\php\\Template\\" => "Ozone\\Framework\\Template\\"
        ];

        return strtr($unique_path, $translations);
    }

    /**
     * Determine whether an account is one of the system-generated accounts.
     * @param int $id
     * @return bool
     */
    public static function isSystemAccount(int $id) : bool
    {
        return ($id === User::ANONYMOUS_USER || $id === User::AUTOMATIC_USER);
    }

}
