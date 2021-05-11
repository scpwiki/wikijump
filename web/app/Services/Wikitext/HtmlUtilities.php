<?php
declare(strict_types = 1);

namespace Wikijump\Services\Wikitext;

class HtmlUtilities
{
    // copied from Wikidot, TODO refactor
    public static function purify(string $html): string
    {
        //$html is not a complete page so we need to wrap it!
        $head = '<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd"><html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en" lang="en">  <head>    <title>Just A Wrapper</title><meta http-equiv="content-type" content="text/html;charset=UTF-8"/>  </head> <!--wrapdelimiter--><body>';
        $tail = ' </body><!--wrapdelimiter--></html>';

        $c = $head . $html . $tail;
        $config = array(
            'indent' => false,
            'output-xhtml' => true,
            'wrap' => 0
        );

        $c2 = tidy_parse_string($c, $config, 'UTF8');

        $arr = explode("<!--wrapdelimiter-->", $c2);
        $out = $arr[1];
        $out = str_replace("<body>", "", $out);
        $out = str_replace("</body>", "", $out);
        return $out;
    }
}