<?php

namespace Unit\Classes;

use Text_Wiki_Parse;
use PHPUnit\Framework\TestCase;

class Text_Wiki_ParseTest extends TestCase
{
    /** @test */
    public function a_parse_object_has_the_required_attributes()
    {
        // We always create a parse object by reference, it doesn't matter for this test what's in the reference.
        $temp = null;
        $parse = new Text_Wiki_Parse($temp);

        $this->assertIsArray($parse->conf);
        $this->assertNull($parse->regex);
        $this->assertNull($parse->wiki);
        $this->assertEmpty($parse->rule);
    }
}
