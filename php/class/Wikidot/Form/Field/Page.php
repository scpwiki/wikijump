<?php


namespace Wikidot\Form\Field;

use Wikidot\Form\Field\Base;



class Page extends Base {
    public function renderView() {
        return "<a href=\"/{$this->data}\">{$this->data}</a>";
    }
}
