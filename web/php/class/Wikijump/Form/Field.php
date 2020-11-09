<?php


namespace Wikijump\Form;

use Wikijump\Form\Field\Text;
use Wikijump\Form\Field\Wiki;
use Wikijump\Form\Field\Select;
use Wikijump\Form\Field\Page;
use Wikijump\Form\Field\PagePath;

class Field
{
    public static function field($field)
    {
        $t = $field['type'];
        if ($t == 'text') {
            return new Text($field);
        }
        if ($t == 'wiki') {
            return new Wiki($field);
        }
        if ($t == 'select') {
            return new Select($field);
        }
        if ($t == 'page') {
            return new Page($field);
        }
        if ($t == 'pagepath') {
            return new PagePath($field);
        }
    }
}
