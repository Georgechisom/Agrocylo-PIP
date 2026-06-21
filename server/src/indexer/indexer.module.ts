import { Module } from '@nestjs/common';
import { SorobanEventListenerService } from './soroban-event-listener.service';

/**
 * Indexer Module
 *
 * Responsible for listening to and processing blockchain events.
 */
@Module({
  providers: [SorobanEventListenerService],
  exports: [SorobanEventListenerService],
})
export class IndexerModule {}
