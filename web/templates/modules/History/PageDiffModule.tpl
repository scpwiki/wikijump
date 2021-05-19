<h2>{t}Page revisions comparison{/t}</h2>
<div class="diff-box">
	<table class="page-compare">
		<tr>
			<th></th>
			<th>{t}Revision{/t} {$fromRevision->getRevisionNumber()}</th>
			<th>{t}Revision{/t} {$toRevision->getRevisionNumber()}</th>
		</tr>
		<tr>
			<td>{t}Created on{/t}:</td>
			<td><span class="odate">{$fromRevision->getDateLastEdited()->getTimestamp()}|%e %b %Y, %H:%M %Z|agohover</span></td>
			<td><span class="odate">{$toRevision->getDateLastEdited()->getTimestamp()}|%e %b %Y, %H:%M %Z|agohover</span></td>
		</tr>
		{if $changed.title}
			<tr>
				<td>{t}Page title{/t}:</td>
				<td>{$fromMetadata->getTitle()|escape}</td>
				<td>{$toMetadata->getTitle()|escape}</td>
			</tr>
		{/if}
		{if $changed.unix_name}
			<tr>
				<td>{t}Page name{/t}:</td>
				<td>{$fromMetadata->getUnixName()|escape}</td>
				<td>{$toMetadata->getUnixName()|escape}</td>
			</tr>
		{/if}
		{if $changed.parent}
			<tr>
				<td>{t}Parent page{/t}:</td>
				<td><a href="/{$fromParent|escape}">{$fromParent|escape}</a></td>
				<td><a href="/{$toParent|escape}">{$toParent|escape}</a></td>
			</tr>
		{/if}
	</table>
	<h3>{t}Source change{/t}:</h3>
	{if $changed.source}
		{if isset($difference)}
			<table class="diff-table">
				<tr>
					<th>{t}from{/t}</th>
					<th>{t}action{/t}</th>
					<th>{t}to{/t}</th>
				</tr>
				{foreach from=$difference item=di}
				<tr>
					{if $di->type=="copy"}
						<td class="from">{foreach from=$di->orig item=tm}{$tm|escape}<br/>{/foreach}</td>
						<td class="action">copy</td>
						<td class="to">{foreach from=$di->final item=tm}{$tm|escape}<br/>{/foreach}</td>
					{/if}
					{if $di->type=="delete"}
						<td class="from">{foreach from=$di->orig item=tm}{$tm|escape}<br/>{/foreach}</td>
						<td class="action">delete</td>
						<td class="to">{foreach from=$di->final item=tm}{$tm|escape}<br/>{/foreach}</td>
					{/if}
					{if $di->type=="add"}
						<td class="from">{foreach from=$di->orig item=tm}{$tm|escape}<br/>{/foreach}</td>
						<td class="action">add</td>
						<td class="to">{foreach from=$di->final item=tm}{$tm|escape}<br/>{/foreach}</td>
					{/if}
					{if $di->type=="change"}
						<td class="from">{foreach from=$di->orig item=tm}{$tm|escape}<br/>{/foreach}</td>
						<td class="action">change</td>
						<td class="to">{foreach from=$di->final item=tm}{$tm|escape}<br/>{/foreach}</td>
					{/if}
				</tr>
				{/foreach}
			</table>
		{/if}
		{if isset($inlineDiff)}
			<div class="inline-diff page-source">
				{$inlineDiff|semipre}
			</div>
		{/if}
	{else}
		<p>
			{t}Page sources are identical.{/t}
		</p>
	{/if}
</div>
