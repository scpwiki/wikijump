<?php
declare(strict_types=1);

namespace Wikijump\Services\Wikitext\FFI;

use Wikijump\Services\Wikitext\ParseRenderMode;

/**
 * Class WikitextSettings, representing an input 'struct ftml_wikitext_settings' object.
 * See Wikijump\Services\Wikitext\WikitextSettings for a version of this class
 * intended for general, non-FFI consumption.
 * @package Wikijump\Services\Wikitext\FFI
 */
class WikitextSettings
{
    private FFI\CData $c_data;

    /**
     * @param Wikitext\WikitextSettings|FFI\CData $settings
     */
    public function __construct($settings)
    {
        if (is_a($settings, 'FFI\CData')) {
            $this->c_data = $settings;
            return;
        }

        $this->c_data = FtmlFfi::make(FtmlFfi::$FTML_WIKITEXT_SETTINGS);
        $this->c_data->mode = ParseRenderMode::toFfiMode($settings->mode);
        $this->c_data->enable_page_syntax = $settings->enable_page_syntax;
        $this->c_data->use_true_ids = $settings->use_true_ids;
        $this->c_data->allow_local_paths = $settings->allow_local_paths;
    }

    public function pointer(): FFI\CData
    {
        return FFI::addr($this->c_data);
    }

    function __destruct()
    {
        FFI::free($this->c_data);
    }
}
