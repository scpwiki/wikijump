
<h1>{t}Files{/t}</h1>



{if isset($files)}
	<table class="page-files">

		<tr>
			<th>{t}file name{/t}</th>
			<th>{t}file type{/t}</th>
			<th>{t}size{/t}</th>
			<th></th>
		</tr>

		{foreach from=$files item=file}
			<tr id="file-row-{$file->getFileId()}">
				<td >
					<a href="{$filePath}{$file->getFilename()|escape:"url"}">{$file->getFilename()|escape}</a>
				</td>
				<td >
					<span title="{$file->getDescription()|escape}">{$file->getDescriptionShort()|escape}</span>
				</td>
				<td >
					{$file->getSizeString()}
				</td>
				{*<td>
					<odate>{$file->getDateAdded()->getTimestamp()}</wbdate>
				</td>*}
				<td>
					<a href="javascript:;" onclick="Wikijump.modules.PageFilesModule.listeners.fileMoreInfo(event, {$file->getFileId()})">info</a>
					|
					<a href="javascript:;" onclick="toggleFileOptions({$file->getFileId()})">{t}options{/t}</a>
				</td>


			</tr>
		{/foreach}
	</table>
	<p>
		{t}Total files size{/t}: {$totalPageSize}
{else}
	<p>
		{t}No files attached to this page.{/t}

	</p>
{/if}

<div style="margin-top:1em">
<a href="javascript:;" id="show-upload-button">{t}upload new file{/t}</a>
| <a href="javascript:;" onclick="Wikijump.modules.PageFilesModule.listeners.fileManager(event)">{t}file manager{/t}</a>
</div>
<div id="file-action-area">
</div>

<div style="display: none" id="file-options-template">
<a href="javascript:;" onclick="Wikijump.modules.PageFilesModule.listeners.renameFile(event)">{t}rename{/t}</a> |
<a href="javascript:;" onclick="Wikijump.modules.PageFilesModule.listeners.moveFile(event)">{t}move{/t}</a> |
<a href="javascript:;" onclick="Wikijump.modules.PageFilesModule.listeners.deleteFile(event)">{t}delete{/t}</a>
</div>
