<?php /* Smarty version 2.6.7, created on 2008-12-06 17:18:19
         compiled from /var/www/wikidot/templates/macros/GlobalMacros.autoload.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('block', 'defmacro', '/var/www/wikidot/templates/macros/GlobalMacros.autoload.tpl', 4, false),)), $this); ?>
 <?php $this->_tag_stack[] = array('defmacro', array('name' => 'printErrorMessages')); smarty_block_defmacro($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?> <?php echo ' 
	{if $messages != null}
	<div style="text-align: center">
	<table class="errormess">
		<tr>
			<td>
				<img src="{$ui->image("warningsymbol.png")}" alt="*"/>
			</td>
			<td class="messholder">
				{foreach from=$messages item="message"}
					{$message}
				{/foreach}
			</td>
		</tr>
	</table>
	</div>
	{/if}
'; ?>
 <?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_defmacro($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?> 

 <?php $this->_tag_stack[] = array('defmacro', array('name' => 'printSuccessMessages')); smarty_block_defmacro($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?> <?php echo ' 
	{if $messages != null}
	<div style="text-align: center">
	<table class="successmess">
		<tr>
			<td>
				<img src="{$ui->image("warning1.png")}" alt="*"/>
			</td>
			<td class="messholder">
		{foreach from=$messages item="message"}
			{$message}
		{/foreach}
			</td>
		</tr>
	</table>
	</div>
	{/if}
'; ?>
 <?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_defmacro($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?> 



 <?php $this->_tag_stack[] = array('defmacro', array('name' => 'pager')); smarty_block_defmacro($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?> <?php echo ' 

{assign var=currentPage value=$pagerData->getCurrentPage()}
{assign var=knownPages value=$pagerData->getKnownPages()}
{assign var=totalPages value=$pagerData->getTotalPages()}
{if $totalPages > 1 || $knownPages>1}
<table class="pager" 
	{if $style}
		style="{$style}"
	{/if}	
>
	<tr>
		<td>
			strona {$currentPage}
			{if $totalPages != null}
				z {$totalPages}
				{assign var=pages value=$totalPages}
			{else}
				{assign var=pages value=$knownPages}
			{/if} 
		</td>
		
		{if $currentPage != 1}
			{assign var="topage" value=$currentPage-1}
			<td>
				<a href="{$golink->copy()->addParameter("page_number", $topage)}"
					><img alt="prev" src="{$ui->image("aleft.gif")}"/></a>
			</td>
		{/if}
		<td>
			{if $currentPage > 3}
				<a href="{$golink->copy()->addParameter("page_number", 1)}">1</a>
			{/if}
			{if $currentPage > 4}
				<a href="{$golink->copy()->addParameter("page_number", 2)}">2</a>
			{/if}


			{if $currentPage  == 6}
				<a href="{$golink->copy()->addParameter("page_number", 3)}">3</a>
			{/if}
			{if $currentPage  > 6}
				...	
			{/if}

			{if $currentPage-2 >= 1}
				{assign var="topage" value=$currentPage-2}
				<a href="{$golink->copy()->addParameter("page_number", $topage)}">{$currentPage-2}</a>
			{/if}

			{if ($currentPage-1) >= 1}
				{assign var="topage" value=$currentPage-1}
				<a href="{$golink->copy()->addParameter("page_number", $topage)}">{$currentPage-1}</a>
			{/if}

			<a class="current" href="{$golink->copy()->addParameter("page_number", $currentPage)}">{$currentPage}</a>

			{if $currentPage+1 <= $pages}
				{assign var="topage" value=$currentPage+1}
				<a href="{$golink->copy()->addParameter("page_number", $topage)}">{$currentPage+1}</a>
			{/if}

			{if $currentPage+2 <= $pages}
				{assign var="topage" value=$currentPage+2}
				<a href="{$golink->copy()->addParameter("page_number", $topage)}">{$currentPage+2}</a>
			{/if}

			{if $currentPage  < $pages-5}
				...
			{/if}

			{if $currentPage  == $pages-5}
				{assign var="topage" value=$tpages-2}
				<a href="{$golink->copy()->addParameter("page_number", $topage)}">{$pages-2}</a>
			{/if}

			{if $currentPage < $pages-3}
				{assign var="topage" value=$pages-1}
				<a href="{$golink->copy()->addParameter("page_number", $topage)}">{$pages-1}</a>
			{/if}
			
			{if $knownPages != null}
				...
			{/if}

			{if $currentPage < $pages-2}
				<a href="{$golink->copy()->addParameter("page_number", $pages)}">{$pages}</a>
			{/if}
		</td>
		
		{if $currentPage != $pages}
			<td>
				{assign var="topage" value=$currentPage+1}
				<a href="{$golink->copy()->addParameter("page_number", $topage)}"
				><img alt="next" src="{$ui->image("aright.gif")}"/></a>
			</td>
		{/if}
	</tr>
</table>

{/if}
'; ?>
 <?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_defmacro($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?> 

 <?php $this->_tag_stack[] = array('defmacro', array('name' => 'printUser')); smarty_block_defmacro($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?> <?php echo ' 
	<a href="{$HTTP_SCHEMA}://{$URL_HOST}/user:info/{$user->getUnixName()}">{$user->getNickName()|escape}</a>
'; ?>
 <?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_defmacro($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?> 
