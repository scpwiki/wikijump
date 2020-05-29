<?php


namespace Wikidot\Form;

use Wikidot\Form\Field\Text;
use Wikidot\Form\Field\Wiki;
use Wikidot\Form\Field\Select;
use Wikidot\Form\Field\Page;
use Wikidot\Form\Field\PagePath;



class Field {
    static public function field($field) {
        $t = $field['type'];
        if ($t == 'text') return new Text($field);
        if ($t == 'wiki') return new Wiki($field);
        if ($t == 'select') return new Select($field);
        if ($t == 'page') return new Page($field);
        if ($t == 'pagepath') return new PagePath($field);
    }
}

