<?php
declare(strict_types=1);

namespace Wikijump\Services\Wikitext;

use Exception;
use Wikijump\Services\Wikitext\FFI\FtmlFfi;

/**
 * Class WikitextSettings, representing information that can influence a call to ftml.
 * @package Wikijump\Services\Wikitext
 */
class WikitextSettings
{
    public int $mode;
    public bool $enable_page_syntax;
    public bool $use_true_ids;
    public bool $allow_local_paths;

    /**
     * Creates a new instance of WikitextSettings.
     *
     * @param int $mode ParseRenderMode value to use
     * @param bool $enable_page_syntax
     * @param bool $use_true_ids
     * @param bool $allow_local_paths
     */
    public function __construct(
        int $mode,
        bool $enable_page_syntax,
        bool $use_true_ids,
        bool $allow_local_paths
    ) {
        $this->mode = $mode;
        $this->enable_page_syntax = $enable_page_syntax;
        $this->use_true_ids = $use_true_ids;
        $this->allow_local_paths = $allow_local_paths;
    }

    public static function fromMode(int $mode): WikitextSettings
    {
        $c_mode = ParseRenderMode::toFfiMode($mode);
        return FtmlFfi::settingsFromMode($c_mode)->toSettings();
    }
}
