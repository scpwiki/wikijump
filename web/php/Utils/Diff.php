<?php

namespace Wikidot\Utils;


use Wikidot\Utils\WDStringUtils;

/**
 * A set of methods handling diff operations.
 *
 */
class Diff
{
    const INLINE_CONTEXT_LINES = 4;

    /**
     * Implementation of unified diff of two strings
     *
     * @param $fromString string the first string to create diff from
     * @param $toString string the second string to create diff from
     * @param $contextLines int the number of lines of context
     * @param $minimal bool whether to find the minimal diff or just any good
     * @return string the unified diff
     */
    public static function unifiedDiff(string $fromString, string $toString, int $contextLines = 3, bool $minimal = false): string
    {
        $file_from = tempnam(WIKIJUMP_ROOT . '/tmp', 'diff-');
        $file_to = tempnam(WIKIJUMP_ROOT . '/tmp', 'diff-');
        file_put_contents($file_from, $fromString);
        file_put_contents($file_to, $toString);

        $from_arg = escapeshellarg($file_from);
        $to_arg = escapeshellarg($file_to);
        $minimal_arg = $minimal ? "-d" : "";
        $context_arg = (int) $contextLines;
        $cmd = "diff $minimal_arg -U $context_arg $from_arg $to_arg";

        $result_lines = array();
        exec($cmd, $result_lines);
        array_shift($result_lines);
        array_shift($result_lines);

        unlink($file_from);
        unlink($file_to);

        return implode("\n", $result_lines);
    }

    /**
     * Generates a difference between two strings.
     *
     * @param string $fromString
     * @param string $toString
     * @return string
     */
    public static function generateStringDiff(string $fromString, string $toString, int $contextLines = 1, bool $minimal = true): string
    {
        // fix "no new line at the end" problem.
        $fromString = WDStringUtils::addTrailingNewline($fromString);
        $toString = WDStringUtils::addTrailingNewline($toString);

        return self::unifiedDiff($fromString, $toString, $contextLines, $minimal);
    }

    /**
     * Generates a nice inline diff.
     *
     * @param string $fromString
     * @param string $toString
     * @return string
     */
    public static function generateInlineStringDiff(string $fromString, string $toString): string
    {
        // make a diff with the FULL output included too.
        $diff = self::generateStringDiff($fromString, $toString, self::INLINE_CONTEXT_LINES);

        $diffLines = explode("\n", $diff);
        $diffs = [];
        for ($i = 0; $i < count($diffLines); $i++) {
            $d = $diffLines[$i];
            if (strlen($d) == 0) {
                continue;
            }

            switch ($d[0]) {
                case ' ':
                    $type = 'copy';
                    break;
                case '-':
                    $type = 'delete';
                    break;
                case '+':
                    $type = 'add';
                    break;
                case '@':
                    $type = 'sep';
                    break;
                default:
                    throw new ProcessException('Invalid line mode from diff: ' . $d[0]);
            }
            array_push($diffs, ['type' => $type, 'line' => substr($d, 1)]);
        }

        // Get rid of leading or trailing separators
        if (count($diffs) > 0) {
            if ($diffs[0]['type'] === 'sep') {
                unset($diffs[0]);
            }

            $last = count($diffs) - 1;
            if ($diffs[$last]['type'] === 'sep') {
                array_pop($diffs);
            }
        }

        // generate output
        $output = [];
        $currentType = 'copy';
        $countDiffs = count($diffs);
        for ($i = 0; $i < $countDiffs; $i++) {
            $row = '';
            $d = $diffs[$i];
            $type = $d['type'];
            if ($type != $currentType) {
                switch ($type) {
                    case 'add':
                        $row .= '<ins>';
                        break;
                    case 'delete':
                        $row .= '<del>';
                        break;
                    case 'sep':
                        array_push($output, '<br><br>');
                }
                $currentType = $type;

                // We don't have a line to output
                if ($currentType === 'sep') {
                    continue;
                }
            }

            $row .= htmlspecialchars($d['line']);

            if ($i<$countDiffs - 1) {
                $nextType = $diffs[$i+1]['type'];
            } else {
                $nextType = null;
            }
            if ($type != $nextType) {
                // close the type
                switch ($type) {
                    case 'add':
                        $row .= '</ins>';
                        break;
                    case 'delete':
                        $row .= '</del>';
                        break;
                }
            }
            array_push($output, $row);
        }

        return implode("\n", $output);
    }
}
