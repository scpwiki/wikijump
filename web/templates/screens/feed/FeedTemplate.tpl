<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0" xmlns:content="http://purl.org/rss/1.0/modules/content/" xmlns:wikijump="http://wikijump.org/rss-namespace">

	<channel>
		<title>{$channel.title|escape}</title>
		<link>{$channel.link|escape}</link>
		<description>{$channel.description|escape}</description>
		{if $channel.language}<language>{$channel.language|escape}</language>{/if}
		<copyright>{$channel.copyright|escape}</copyright>
		<lastBuildDate>{$channel.lastBuildDate|escape}</lastBuildDate>

		{foreach from=$items item=item}
			<item>
				<guid>{$item.guid|escape}</guid>
				<title>{$item.title|escape}</title>
				<link>{$item.link|escape}</link>
				<description>{$item.description|escape}</description>
				<pubDate>{$item.date|escape}</pubDate>
				{if $item.author}<wikijump:authorName>{$item.author|escape}</wikijump:authorName>{/if}
				{if $item.authorUserId}<wikijump:authorUserId>{$item.authorUserId|escape}</wikijump:authorUserId>{/if}
				{if $item.content}<content:encoded>
					<![CDATA[
						{$item.content|strip}
				 	]]>
				</content:encoded>{/if}
				{if $item.category}
				<category>{$item.category|escape}</category>
				{/if}
			</item>
		{/foreach}
		</channel>
</rss>
