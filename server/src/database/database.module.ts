import { Global, Module, OnModuleDestroy } from '@nestjs/common';
import { PrismaClient } from '../../generated/prisma/client';
import { PrismaLibSql } from '@prisma/adapter-libsql';

@Global()
@Module({
  providers: [
    {
      provide: PrismaClient,
      useFactory: () => {
        const url = process.env.DATABASE_URL || 'file:./dev.db';
        const adapter = new PrismaLibSql({ url });
        return new PrismaClient({ adapter });
      },
    },
  ],
  exports: [PrismaClient],
})
export class DatabaseModule implements OnModuleDestroy {
  constructor(private readonly prisma: PrismaClient) {}

  async onModuleDestroy() {
    await this.prisma.$disconnect();
  }
}
