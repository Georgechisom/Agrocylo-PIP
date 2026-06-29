import { Module } from '@nestjs/common';
import { SorobanEventListenerService } from './soroban-event-listener.service';
import { EventParserService } from './parsers/event-parser.service';

@Module({
  providers: [SorobanEventListenerService, EventParserService],
  exports: [SorobanEventListenerService, EventParserService],
})
export class IndexerModule {}
