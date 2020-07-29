<?php


namespace Wikijump\Form\Field;

use \WDStringUtils;
use Criteria;
use DB\CategoryPeer;
use DB\PagePeer;

class PagePath extends WikiBase
{
    public $rule = '/
        ^
        (
            \[\[\[     # Exactly 3 opening brackets
            [^|]]*     # Not sure. Might be an error here. Escape is needed
            (|[^]]*)?  # Nothing, or anything that is not a closing bracket?
            \]\]\]     # Exactly 3 closing brackets
            [\s\/]*
        )*$
        /x';
    # I get the impression that this is supposed to be Text_Wiki extension
    # for triple-bracket intra-wiki URLs, but that regex there is not correct.
    # I suspect the real logic is defined elsewhere.
    # We will have to investigate whether or not this function is even called.
    public function renderEdit()
    {
        $m = array();
        $path = array();
        $v = $this->field['value'];

        $path = array();
        if (preg_match($this->rule, $v, $m)) {
            $parts = explode(']]]', $v);
            foreach ($parts as $part) {
                $m = array();
                if (preg_match(
                    '/
                    ^        # Start of text
                    [^[]*    # Anything that is not [
                    \[\[\[   # Three opening brackets
                    ([^|]*)  # Anything that is not | (i.e. link location)
                    ([|]|$)  # Terminate at a | or end of text
                    /x',
                    $part,
                    $m
                )) {
                    $path[] = WDStringUtils::toUnixName($m[1]);
                }
            }
        }
        $path[] = '';

        $selects = array();

        $c = new Criteria();
        $c->add('name', $this->field['category']);

        if ($category = CategoryPeer::instance()->selectOne($c)) {
            $categoryId = $category->getCategoryId();
            $pages = array();
            $parentId = null;

            foreach ($path as $part) {
                $select = "<select>";
                $select .= '<option value=""></option>';

                $pages = $this->selectPagesByParent($categoryId, $parentId);
                $parentId = null;
                foreach ($pages as $page) {
                    $unixName = htmlspecialchars($page->getUnixName());
                    $title = htmlspecialchars($page->getTitleOrUnixName());
                    $selected = "";
                    if ($unixName == $part) {
                        $selected = ' selected="selected"';
                        $parentId = $page->getPageId();
                    }
                    $select .= "<option value=\"$unixName\"$selected>$title</option>";
                }
                $select .= '<option value="+" style="border-top: 1px solid #666; font-weight: bold">Create new</option>';
                $select .= '</select>';
                $selects[] = $select;
                if (! $parentId) {
                    break;
                }
            }
        }

        $selectsEnd = '';
        $selectsNo = count($selects);
        for ($i = 1; $i < count($selects); $i++) {
            $selectsEnd .= '</span>';
        }

        return '<div class="field-pagepath-chooser">' .
            '<input class="value" type="hidden" name="field_' . $this->field['name'] . '" value="' . $this->hvalue() . '"/>' .
            '<input class="category" type="hidden" value="' . $this->field['category'] . '"/>' .
            '<input class="new_page_parent" type="hidden" name="newpageparent_' . $this->field['name'] . '" value=""/>' .
            '<input class="new_page_title" type="hidden" name="newpagetitle_' . $this->field['name'] . '" value=""/>' .
            '<span>' . implode("<span> / ", $selects) . '<span></span>' . $selectsEnd . '</span>' .
        '</div>';
        /*
        in the end we get something like this:
            <div class="field-pagepath-choser">
                <input type="hidden" value="" name=""/>
                <span>
                    <select> <option/> <option/> ... </select> <span>
                        / <select> <option/> <option/> ... </select> <span>
                            / <select> <option/> <option/> ... </select> <span>
                                / <select> <option/> <option/> ... </select> <span>
                                </span>
                            </span>
                        </span>
                    </span>
                </span>
            </div>
        */
    }
    public function selectPagesByParent($categoryId, $parentId)
    {
        $c = new Criteria();
        $categoryId = (int) $categoryId;
        $parentId = (int) $parentId;
        if ($parentId) {
            $c->setExplicitQuery("SELECT * FROM page WHERE category_id = $categoryId AND parent_page_id = $parentId");
        } else {
            $c->setExplicitQuery("SELECT * FROM page WHERE category_id = $categoryId AND parent_page_id IS NULL");
        }
        return PagePeer::instance()->select($c);
    }
}
