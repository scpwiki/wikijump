<?php


namespace Wikidot\Form\Field;


class Page extends Base {
    public function renderView() {
        return "<a href=\"/{$this->data}\">{$this->data}</a>";
    }
}
