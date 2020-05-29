<h1>{t}File Information{/t}</h1>

{if $file->getHasResized()}
	<div style="text-align: center; margin: 1em; height: 240px;">
		<img src="{$file->getResizedURI("small")}" alt="thumbnail"/>
	</div>
{/if}

<table>
	<tr>
		<td>
			{t}File name{/t}:
		</td>
		<td>
			<strong>{$file->getFilename()|escape}</strong>
		</td>
	</tr>
	<tr>
		<td>
			{t}Full file URL{/t}:
		</td>
		<td>
			<a href="{$file->getFileURI()|escape}">{$file->getFileURI()|escape}</a>
		</td>
	</tr>
	<tr>
		<td>
			{t}File size{/t}:
		</td>
		<td>
			{$file->getSizeString()} {if $file->getSize()>1024}({$file->getSize()} Bytes){/if} 
		</td>
	</tr>
	<tr>
		<td>
			{t}MIME type{/t}:
		</td>
		<td>
			{$file->getMimetype()}
		</td>
	</tr>
	<tr>
		<td>
			{t}Content type{/t}:
		</td>
		<td>
			{$file->getDescription()}
		</td>
	</tr>
	<tr>
		<td>
			{t}Uploaded by{/t}:
		</td>
		<td>
			{printuser user=$file->getUserOrString() image=true}
		</td>
	</tr>
	<tr>
		<td>
			{t}Upload date{/t}:
		</td>
		<td>
			<span class="odate">{$file->getDateAdded()->getTimestamp()}|%e %b %Y, %H:%M %Z|agohover</span>
		</td>
	</tr>
	{if $file->getComment() && $file->getComment() != ''}
		<tr>
			<td>
				{t}File comment{/t}:
			</td>
			<td>
				{$file->getComment()}
			</td>
		</tr>
	{/if}
</table>